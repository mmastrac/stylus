const Babel = await import('babel');

// Turn a module into a tree of transpiled modules.
export async function importBabel(cache, entryPath, { presets = [] } = {}) {
  const moduleCache = new Map();
  const modulePathMap = new Map();

  // Define a temporary global helper for rewritten imports
  async function importBabelInternal(fullPath) {
    if (moduleCache.has(fullPath)) {
      return moduleCache.get(fullPath);
    }

    const { rewritten, imports } = await transpileModule(modulePathMap, fullPath, presets);
    const blob = new Blob([rewritten], { type: 'application/javascript' });
    const blobUrl = URL.createObjectURL(blob);
    modulePathMap.set(fullPath, blobUrl);

    const timeout = setTimeout(() => {
      console.log('Timeout waiting for import:', fullPath, 'Circular dependency?', 'blobpath =', blobUrl);
    }, 10_000);
    const promise = import(blobUrl).finally(() => clearTimeout(timeout));
    moduleCache.set(fullPath, promise);

    return promise;
  };

  window.__importBabelInternal = importBabelInternal;
  try {
    const module = await importBabelInternal(normalizePath(location.href, entryPath));
    console.log('Done.');
    return module;
  } finally {
    // Clean up
    delete window.__importBabelInternal;
  }
}

async function transpileModule(modulePathMap, fullPath, presets) {
  console.log('Loading and transpiling:', fullPath);

  const raw = await fetch(fullPath).then(res => {
    if (!res.ok) throw new Error(`Failed to fetch ${fullPath}`);
    return res.text();
  });

  const imports = new Set();
  const rewritten = Babel.transform(raw, {
    presets,
    filename: fullPath,
    sourceFileName: fullPath,
    parserOpts: { sourceType: 'module', plugins: ['importAssertions'] },
    plugins: [rewriteStaticImports(modulePathMap, imports, fullPath)],
    sourceType: 'module',
    sourceMaps: "inline",
  }).code;

  return { rewritten, imports };
}

// Normalize to ensure ./ prefix for relative paths
function normalizePath(base, relative) {
  return new URL(relative, base).href;
}

function isRelativeImport(path) {
  return path.startsWith('./') || path.startsWith('../');
}

function rewriteStaticImports(modulePathMap, imports, parentUrl) {
  return ({ types: t }) => ({
    visitor: {
      ImportDeclaration(path) {
        const source = path.node.source.value;
        if (!isRelativeImport(source)) return;

        const fullPath = normalizePath(parentUrl, source);
        imports.add(fullPath);

        const blobUrl = modulePathMap.get(fullPath);
        if (blobUrl) {
          path.node.source.value = blobUrl;
          return;
        }

        const newCall = t.awaitExpression(
          t.callExpression(t.identifier('__importBabelInternal'), [
            t.stringLiteral(fullPath),
          ])
        );

        const specifiers = path.node.specifiers;
        if (specifiers.length === 0) {
          path.replaceWith(t.expressionStatement(newCall));
          return;
        }

        const bindings = [];
        for (const spec of specifiers) {
          if (t.isImportDefaultSpecifier(spec)) {
            bindings.push(t.objectProperty(t.identifier('default'), spec.local));
          } else if (t.isImportSpecifier(spec)) {
            bindings.push(
              t.objectProperty(
                spec.imported,
                spec.local,
                false,
                spec.imported.name === spec.local.name
              )
            );
          } else if (t.isImportNamespaceSpecifier(spec)) {
            path.replaceWith(
              t.variableDeclaration('const', [
                t.variableDeclarator(spec.local, newCall),
              ])
            );
            return;
          }
        }

        path.replaceWith(
          t.variableDeclaration('const', [
            t.variableDeclarator(t.objectPattern(bindings), newCall),
          ])
        );
      },

      ExportNamedDeclaration(path) {
        // Leave plain exports alone: export { a, b }
        const src = path.node.source && path.node.source.value;
        if (!src) return;
      
        // Only rewrite relative sources
        if (!isRelativeImport(src)) return;
      
        // Skip type-only re-exports (TS/Flow)
        if (path.node.exportKind === 'type') return;
      
        const fullPath = normalizePath(parentUrl, src);
        imports.add(fullPath);

        const blobUrl = modulePathMap.get(fullPath);
        if (blobUrl) {
          path.node.source.value = blobUrl;
          return;
        }

        const importCall = t.awaitExpression(
          t.callExpression(t.identifier('__importBabelInternal'), [
            t.stringLiteral(fullPath),
          ])
        );
      
        // Build: const { local1: _u1, local2: _u2 } = await import(...);
        const destructProps = [];
        const exportSpecs = [];
      
        for (const spec of path.node.specifiers) {
          if (!t.isExportSpecifier(spec)) continue;
      
          // `local` is the name in the source module (can be Identifier or StringLiteral; default is Identifier('default'))
          const localKey = spec.local;         // Identifier | StringLiteral
          const exported = spec.exported;      // Identifier | StringLiteral
      
          const uid = path.scope.generateUidIdentifier(
            t.isIdentifier(exported) ? exported.name : 'reexp'
          );
      
          // { localKey: uid }
          // (non-computed, key must be Identifier or StringLiteral)
          if (t.isIdentifier(localKey) || t.isStringLiteral(localKey)) {
            destructProps.push(t.objectProperty(localKey, uid, false, false));
          } else {
            // Extremely rare fallback; normalize to string key
            destructProps.push(t.objectProperty(t.stringLiteral(String(localKey.name)), uid));
          }
      
          // export { uid as exported }
          exportSpecs.push(t.exportSpecifier(uid, exported));
        }
      
        if (destructProps.length === 0) {
          // Nothing to rewrite; keep as-is.
          return;
        }
      
        const tmpDecl = t.variableDeclaration('const', [
          t.variableDeclarator(t.objectPattern(destructProps), importCall),
        ]);
      
        const newExport = t.exportNamedDeclaration(null, exportSpecs, null);
      
        path.replaceWithMultiple([tmpDecl, newExport]);
      },
      
      ExportAllDeclaration(path) {
        // Still unsupported: cannot expand statically without enumerating exports.
        throw path.buildCodeFrameError(
          'export * from ... is not supported by importBabel yet.'
        );
      },

      CallExpression(path) {
        if (path.node.callee.type === 'Import') {
          const arg = path.node.arguments[0];
          if (t.isStringLiteral(arg) && isRelativeImport(arg.value)) {
            imports.add(arg.value);
            path.node.arguments = [
              t.callExpression(t.identifier('__importBabelInternal'), [
                arg,
                t.stringLiteral(parentUrl),
              ]),
            ];
          }
        }
      },
    },
  });
}
