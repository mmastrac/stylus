const LOCAL_STORAGE_KEY = 'stylus-root-';

class FetchAwareCache {
    constructor(key, friendly) {
        this.key = key;
        this.friendly = friendly;
    }

    async get(subkey, builder, applyEarly = null) {
        const cache = this._parseLocalStorage(this.key + subkey);
        if (cache && cache.files && cache.result) {
            if (applyEarly) {
                applyEarly(cache.result);
            }

            // If we have a cache, check the cache status for the files
            const files = cache.files;
            const fetches = [];
            for (const file of files) {
                fetches.push(fetch(file, { mode: "same-origin", cache: "only-if-cached" }).catch(() => ({
                    status: 500
                })));
            }

            console.log(this.friendly, ': Checking freshness of', files.length, 'file(s)');

            const results = await Promise.all(fetches);
            console.log(this.friendly, ': Results', results);
            let fresh = true;
            for (const result of results) {
                if (result.status !== 200) {
                    fresh = false;
                    break;
                }
            }

            if (fresh) {
                return cache.result;
            }
        }

        const result = await builder();
        localStorage[this.key + subkey] = JSON.stringify(result);
        return result;
    }

    _parseLocalStorage(key) {
        try {
            const json = localStorage[key] || 'null';
            const parsed = JSON.parse(json);
            if (!parsed) {
                delete localStorage[key];
                return null;
            }
            return parsed;
        } catch (e) {
            delete localStorage[key];
            return null;
        }
    }
}

async function loadRoot() {
    const res = await fetch('import_map.json');
    const json = await res.json();
    const script = document.createElement('script');
    script.type = 'importmap';
    script.textContent = JSON.stringify(json);
    document.getElementsByTagName('head')[0].insertBefore(script, null);

    const module = await import('./babel-module-loader.js');
    await module.importBabel(
        new FetchAwareCache(LOCAL_STORAGE_KEY + 'babel-cache', 'BABEL'),
        './src/app.tsx', 
        { presets: [['react', { runtime: 'automatic' }], 'typescript'] }
    );
}

loadRoot().catch(error => {
    console.error('Failed to load application:', error);
    document.getElementById('root').innerHTML = `
        <div style="padding: 20px; color: red;">
            <h2>Failed to load application</h2>
            <p style="white-space: pre-wrap; font-family: monospace;">${error.message}</p>
        </div>
    `;  
});