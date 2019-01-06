'use strict';
import comphdl from "../comphdl_web/Cargo.toml";
import * as ace from 'brace';
import 'brace/theme/tomorrow';
import 'brace/mode/rust';
import { Terminal } from 'xterm';
import * as WaveDromFastUpdate from './wavedrom_fast_update.js'
//import { WaveDrom } from 'wavedrom';
window.WaveSkin = require('wavedrom/skins/narrow.js');
var WaveDrom = require('wavedrom');
var Stats = require('stats.js');

var stats = new Stats();
stats.showPanel( 0 );
stats.dom.style.cssText = '';
document.getElementById('statsDiv').appendChild( stats.dom );

// Assets
// import...

ace.config.set('basePath', 'ace-builds/src-noconflict/ace.js')
var editor = ace.edit("comphdl_definition");
editor.setTheme("ace/theme/tomorrow");
editor.session.setMode("ace/mode/rust");

var superagent = require('superagent');
var json5 = require('json5');
var netlistSvg = require('netlistsvg');

var skins = [
    { url: require('./skins/comphdl.svg'), name: 'comphdl.svg' },
];

var textarea = document.querySelector('#comphdl_json');
var skinSelect = document.querySelector('#skinSelect');
var renderButton = document.querySelector('#renderButton');
var formatButton = document.querySelector('#formatButton');
var viewer = document.querySelector('#viewerContainer');

skins.forEach(function(skin, i) {
    superagent.get(skin.url).end(function(err, r) {
        var option = document.createElement('option');
        option.selected = i === 0;
        option.value = r.text;
        option.text = skin.name;
        skinSelect.append(option);
    });
});

function render() {
    var netlist = json5.parse(textarea.value);
    return netlistSvg.render(skinSelect.value, netlist).then(function(svg) {
        viewer.innerHTML = svg;
    });
}

function format() {
    var netlist = json5.parse(textarea.value);
    textarea.value = json5.stringify(netlist, null, 4);
}

renderButton.onclick = render;
formatButton.onclick = format;

// Force this checkbox to false because otherwise we wont be able to press RUN
document.getElementById("check_alive").checked = false;

// Create terminals
const termConfig = {
    cols: 80,
    rows: 24,
    scrollback: 10000, // default is 1000
};
export var term = new Terminal(termConfig);
term.open(document.getElementById('terminal_1'));
term.on('data', (data) => {
  document.getElementById('stdin_bufread').value += data;
})
export var term2 = new Terminal(termConfig);
term2.open(document.getElementById('terminal_2'));
//term2.write('This terminal will be used for logging');
var num_terminals = 2;
function terminalTab(i) {
    var i = Number.parseInt(i);
    for(var j=1; j<=num_terminals; j++) {
        var t = document.getElementById('terminal_' + j);
        if(j == i) {
            t.style.display = 'block';
        } else {
            t.style.display = 'none';
        }
    }
}
terminalTab(0);

document.getElementById("loading_wasm").style.display = "none";
document.getElementById("loaded_wasm").style.display = "block";

/* Parse GET params: index.html?top=TopName&code=asdf */
// TODO: we should do this immediately on window load, without
// waiting for the Rust module
// Also, more options
var url = new URL(window.location.href);
var searchParams = new URLSearchParams(url.search);
var gotCode = searchParams.get('code');  // outputs "m2-m3-m4-m5"
var gotExampleName = searchParams.get('example');

if (gotCode != null) {
    // Ideally this would create a new tab so we dont lose the existing code
    editor.setValue(gotCode);
} else if (gotExampleName != null) {
    loadExample(false, gotExampleName);
} else {
    loadExample(false, "example1.txt");
}

var gotTop = searchParams.get('top');  // outputs "m2-m3-m4-m5"
if (gotTop != null) {
    document.getElementById("top_name").value = gotTop;
}

var examples = ["example1.txt", "bufbufbuf.txt", "ram.txt", "srlatch.txt", "cat.txt"];
if (gotExampleName == null) {
    gotExampleName = "example1.txt";
}
var examplesSelect = document.getElementById("exampleName");
examples.forEach(function(e, i) {
    var option = document.createElement('option');
    option.selected = e == gotExampleName;
    option.value = e;
    option.text = e;
    examplesSelect.append(option);
});
dragElement(document.getElementById("draggableControls"));

export function register_main_loop(main_loop) {
        var check_run_forever = document.getElementById("check_run_forever");
        var check_run_step = document.getElementById("check_run_step");
        var check_alive = document.getElementById("check_alive");
        var tick_display = document.getElementById("tick_display");
        var check_show_debug = document.getElementById("check_show_debug");
        var check_show_signals = document.getElementById("check_show_signals");
        var check_render_wavedrom = document.getElementById("check_render_wavedrom");
        var check_monitor_signals = document.getElementById("check_monitor_signals");
        var target_ticks_per_second = document.getElementById("target_ticks_per_second");
        var tick = 0;
        var intervalId;

        function demo() {
            if(check_run_forever.checked || check_run_step.checked) {
                stats.begin();
                main_loop(check_show_debug.checked, check_show_signals.checked, check_monitor_signals.checked);
                stats.end();
                if(check_render_wavedrom.checked) {
                    // FIXME: we must force tick to be 0 until the graph is full
                    // To fix, find a way to preallocate an empty graph or just
                    // set everything to X
                    WaveDromFastUpdate.refreshWaveDrom(WaveDrom, tick > 50 ? tick : 0);
                }
                check_run_step.checked = false;
                tick += 1;
                tick_display.value = tick;
            }

            if(check_alive.checked == false) {
                // Stop running
                main_loop.drop(); // Necessary to clean up the closure on Rust's side.
                clearInterval(intervalId);
            } else {
                /*
                // Can we use setInterval if the function takes more than 1000/30 ms
                // to run? Yes, js is singlethreaded.
                var fps = parseInt(target_ticks_per_second.value, 10);
                if(isNaN(fps)) { fps = 30; }
                setTimeout(demo, 1000/fps);
                */
            }
        }

        var target_tps = 30;
        if (target_ticks_per_second) {
            var x = Number.parseFloat(target_ticks_per_second.value);
            if (x) {
                target_tps = x;
            }
        }
        console.log('setInterval to ' + target_tps);
        intervalId = setInterval(demo, 1000/target_tps);
}

function loadExampleSelect() {
    var name = document.getElementById("exampleName").value;
    stopSimulation();
    loadExample(true, name);
}

function loadExample(force, name) {
    var editorValue = editor.getValue();
    // Do not overwrite existing code
    if (force == false && editorValue != "") {
        return;
    }
    fetch("./comphdl_examples/" + name).then(function(response) {
        if (!response.ok) {
            throw Error(response.statusText);
        }
        return response;
    }).then(function(response) {
        response.text().then(function(text) {
            editor.setValue(text);
        });
    });
}

function shareCode() {
    var URLWithoutParams = [window.location.protocol, '//', window.location.host, window.location.pathname].join('');

    var code = editor.getValue();
    var codeURI = encodeURIComponent(code);

    var topName = document.getElementById("top_name").value;
    var topNameURI = encodeURIComponent(topName);

    var fullURL = URLWithoutParams + '?top=' + topNameURI + '&code=' + codeURI;

    share_div = document.getElementById("shareCodeDiv");
    share_div.style.display = "block";
    share_input = document.getElementById("shareCodeLink");
    share_input.value = fullURL;
}

function runGui() {
    var alive = document.getElementById("check_alive");
    // TODO: check if a thread is already running and kill it
    if (alive.checked) {
        // TODO: kill it
        alert("Stop the program before running it again");
        showSimulationControls();
        // We cannot kill it here because the Rust code will not update
        // and we get a classical race condition.
        // This could maybe be fixed by giving each Rust instance its own
        // uuid, and have a field "running_uuid" which must match or the
        // program will stop
        return;
    }

    pauseSimulation();
    alive.checked = true;

    var error_string = comphdl.run_js_gui(editor.getValue());
    console.log(error_string);
    document.getElementById("top_output_debug").value = error_string;
    showSimulationControls();
    document.getElementById("renderButton").onclick().then(function(svg) {
        // Enable clicking to svg ports toggles inputs
        var ti = document.getElementById("top_input");
        for(var i=0; i<ti.children.length; i++) {
            var tic = ti.children[i];
            var inputid = tic.id.replace("checkbox_input_", "inputExt_");
            tic.onclick = function(){
                var inputid = this.id.replace("checkbox_input_", "inputExt_");
                var ig = document.getElementById(inputid);
                if(ig) {
                    if(this.checked) {
                        ig.style.fill = "var(--comphdl-wire-color-H)";
                    } else {
                        ig.style.fill = "var(--comphdl-wire-color-L)";
                    }
                } else {
                    ig.style.fill = "var(--comphdl-wire-color-X)";
                }
            };
            var ig = document.getElementById(inputid);
            // Initialize inputExt to 0
            if(ig) {
                ig.style.fill = "var(--comphdl-wire-color-L)";
                ig.onclick = function(){
                    var checkboxid = this.id.replace("inputExt_", "checkbox_input_");
                    document.getElementById(checkboxid).click();
                };
            } else {
                console.error(tic.id + " is missing the corresponding " + inputid + " in the svg")
            }
        }

        resumeSimulation();
    });
}

function resumeSimulation() {
    var a = document.getElementById("check_run_forever");
    a.checked = true;
}

function pauseSimulation() {
    var a = document.getElementById("check_run_forever");
    a.checked = false;
}

function stopSimulation() {
    var a = document.getElementById("check_alive");
    a.checked = false;
}

function runStep() {
    pauseSimulation();
    var a = document.getElementById("check_run_step");
    a.checked = true;
}

function hideSimulationControls() {
    document.getElementById("simulation_controls").style.display = "none";
}

function showSimulationControls() {
    document.getElementById("simulation_controls").style.display = "block";
}

function toggleFloatingControls() {
    var a = document.getElementById("draggableControls");
    if (a.style.position == 'absolute') {
        a.style.position = 'static';
        a.style.width = '';
    } else {
        a.style.position = 'absolute';
        a.style.width = '500px';
        a.style.zIndex = '10';
    }
}

function dragElement(elmnt) {
    var pos1 = 0,
        pos2 = 0,
        pos3 = 0,
        pos4 = 0;
    if (document.getElementById(elmnt.id + "header")) {
        /* if present, the header is where you move the DIV from:*/
        document.getElementById(elmnt.id + "header").onmousedown = dragMouseDown;
    } else {
        /* otherwise, move the DIV from anywhere inside the DIV:*/
        elmnt.onmousedown = dragMouseDown;
    }

    function dragMouseDown(e) {
        e = e || window.event;
        // get the mouse cursor position at startup:
        pos3 = e.clientX;
        pos4 = e.clientY;
        document.onmouseup = closeDragElement;
        // call a function whenever the cursor moves:
        document.onmousemove = elementDrag;
    }

    function elementDrag(e) {
        e = e || window.event;
        // calculate the new cursor position:
        pos1 = pos3 - e.clientX;
        pos2 = pos4 - e.clientY;
        pos3 = e.clientX;
        pos4 = e.clientY;
        // set the element's new position:
        elmnt.style.top = (elmnt.offsetTop - pos2) + "px";
        elmnt.style.left = (elmnt.offsetLeft - pos1) + "px";
    }

    function closeDragElement() {
        /* stop moving when mouse button is released:*/
        document.onmouseup = null;
        document.onmousemove = null;
    }
}

document.getElementById('comphdl_start').onclick = runGui;
document.getElementById('pause_simulation').onclick = pauseSimulation;
document.getElementById('run_step').onclick = runStep;
document.getElementById('run_forever').onclick = resumeSimulation;
document.getElementById('stop_simulation').onclick = stopSimulation;
document.getElementById('loadExampleButton').onclick = loadExampleSelect;
document.getElementById('shareCode').onclick = shareCode;
document.getElementById('toggleFloatingControls').onclick = toggleFloatingControls;
document.getElementById('refresh_wavedrom').onclick = function() {
    WaveDromFastUpdate.refreshWaveDrom(WaveDrom, 0)
};
document.getElementById('terminal_tab_0').onclick = function() { terminalTab(0) };
document.getElementById('terminal_tab_1').onclick = function() { terminalTab(1) };
document.getElementById('terminal_tab_2').onclick = function() { terminalTab(2) };

