'use strict';
var superagent = require('superagent');
var json5 = require('json5');
var netlistSvg = require('./lib');

var skins = ['skins/default.svg', 'skins/analog.svg'];

var textarea = document.querySelector('#comphdl_json');
var skinSelect = document.querySelector('#skinSelect');
var renderButton = document.querySelector('#renderButton');
var formatButton = document.querySelector('#formatButton');
var viewer = document.querySelector('#viewerContainer');

skins.forEach(function(skinPath, i) {
    superagent.get(skinPath).end(function(err, r) {
        var option = document.createElement('option');
        option.selected = i === 0;
        option.value = r.text;
        option.text = skinPath;
        skinSelect.append(option);
    });
});

function render() {
    var netlist = json5.parse(textarea.value);
    netlistSvg.render(skinSelect.value, netlist, function(e, svg) {
        viewer.innerHTML = svg;
    });
}

function format() {
    var netlist = json5.parse(textarea.value);
    textarea.value = json5.stringify(netlist, null, 4);
}

renderButton.onclick = render;
formatButton.onclick = format;
