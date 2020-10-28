// fetch doesn't work over file:// even when we use --allow-file-access-from-files,
// so override it with a polyfill that uses XHR instead for local debugging.
function fetch(url, options) {
    return new Promise(function(resolve, reject){
        var xhr = new XMLHttpRequest();
        xhr.responseType = "arraybuffer";
        xhr.addEventListener("error", function(err) {
            debugger;
            reject(err);
        });
        xhr.addEventListener("load", function(load) {
            var headers = new Headers();
            xhr.getAllResponseHeaders().split(/[\r\n]+/).forEach(function(header) {
                if (header !== "") {
                    var [key, val] = header.split(": ", 2);
                    console.log(key, val);
                    headers.set(key, val);
                }
            });
            if (url.endsWith(".wasm")) {
                headers.set("Content-Type", "application/wasm");
            }
            resolve(new Response(xhr.response, { status: load.status, headers: headers}));
        });
        xhr.open("GET", url);
        xhr.send();
    })
}
