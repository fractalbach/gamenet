var Module = {
  preRun: [],
  postRun: [],
  print: function(text) {
    if (arguments.length > 1) text =
      Array.prototype.slice.call(arguments).join(' ');
    console.log(text);
  },
  printErr: function(text) {
    if (arguments.length > 1) {
      text = Array.prototype.slice.call(arguments).join(' ');
    }
    console.error(text);
  },
  canvas: function() {
    return undefined;
  },
  setStatus: function(text) {
    console.log('Module status update: ' + text);
  },
  totalDependencies: 0,
  ready: false,
  monitorRunDependencies: function(left) {
    this.totalDependencies = Math.max(this.totalDependencies, left);
    if (left) {
      Module.setStatus('Preparing... (' + (this.totalDependencies-left) +
            '/' + this.totalDependencies + ')');
    } else {
      Module.ready = true;
      Module.setStatus('All downloads complete.');
    }
  }
};