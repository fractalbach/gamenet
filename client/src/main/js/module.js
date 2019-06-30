var procede = {}

procede.ready = false;

WebAssembly.instantiateStreaming(fetch('procede_bg.wasm'), {})
.then(results => {
    procede.exports = results.instance.exports;
    procede.ready = true;
});
