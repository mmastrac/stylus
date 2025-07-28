// Turn a module into a tree of transpiled modules.
export async function importBabel(entryPath, { presets = [] } = {}) {
  const Babel = await import('babel');

  // Normalize to ensure ./ prefix for relative paths
  function normalizePath(base, relative) {
    return new URL(relative, base).href;
  }

  function isRelativeImport(path) {
    return path.startsWith('./') || path.startsWith('../');
  }

  const moduleCache = new Map();

  async function transpileModule(path, parentPath) {
    const fullPath = normalizePath(parentPath || location.href, path);

    if (moduleCache.has(fullPath)) {
      return moduleCache.get(fullPath);
    }

    console.log('Loading and transpiling:', fullPath);

    const raw = await fetch(fullPath).then(res => {
      if (!res.ok) throw new Error(`Failed to fetch ${fullPath}`);
      return res.text();
    });

    const rewritten = Babel.transform(raw, {
      presets,
      filename: fullPath,
      sourceFileName: fullPath,
      parserOpts: { sourceType: 'module', plugins: ['importAssertions'] },
      plugins: [rewriteStaticImports(fullPath)],
      sourceType: 'module',
      sourceMaps: "inline",
    }).code;

    const blob = new Blob([rewritten], { type: 'application/javascript' });
    const blobUrl = URL.createObjectURL(blob);

    moduleCache.set(fullPath, blobUrl);
    return blobUrl;
  }

  function rewriteStaticImports(parentUrl) {
    return ({ types: t }) => ({
      visitor: {
        ImportDeclaration(path) {
          const source = path.node.source.value;
          if (!isRelativeImport(source)) return;
        
          const newCall = t.awaitExpression(
            t.callExpression(t.identifier('__importBabelInternal'), [
              t.stringLiteral(source),
              t.stringLiteral(parentUrl),
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
              bindings.push(t.objectProperty(
                spec.imported,
                spec.local,
                false,
                spec.imported.name === spec.local.name
              ));
            } else if (t.isImportNamespaceSpecifier(spec)) {
              path.replaceWith(t.variableDeclaration('const', [
                t.variableDeclarator(spec.local, newCall)
              ]));
              return;
            }
          }
        
          path.replaceWith(t.variableDeclaration('const', [
            t.variableDeclarator(t.objectPattern(bindings), newCall)
          ]));
        },

        ExportAllDeclaration(path) {
          // unsupported for now — could add logic to re-export from other modules if needed
          throw path.buildCodeFrameError('export * is not supported by importBabel yet.');
        },

        ExportNamedDeclaration(path) {
          if (path.node.source) {
            throw path.buildCodeFrameError('export ... from is not supported by importBabel yet.');
          }
        },

        CallExpression(path) {
          // Handle dynamic import("...")
          if (path.node.callee.type === 'Import') {
            const arg = path.node.arguments[0];
            if (t.isStringLiteral(arg) && isRelativeImport(arg.value)) {
              path.node.arguments = [
                t.callExpression(t.identifier('__importBabelInternal'), [
                  arg,
                  t.stringLiteral(parentUrl),
                ]),
              ];
            }
          }
        }
      }
    });
  }

  // Define a temporary global helper for rewritten imports
  window.__importBabelInternal = async function (relPath, parent) {
    const url = await transpileModule(relPath, parent);
    return await import(url);
  };

  const entryBlobUrl = await transpileModule(entryPath);
  console.log('Importing root module:', entryBlobUrl);
  const module = await import(entryBlobUrl);
  console.log('Done.');

  // Clean up
  delete window.__importBabelInternal;

  return module;
}
