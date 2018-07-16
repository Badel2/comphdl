Rust.comphdl.then(function(comphdl) {
    document.getElementById("loading_wasm").style.display = "none";
    document.getElementById("loaded_wasm").style.display = "block";

    loadExample(false, "example1.txt");
    var examples = ["example1.txt", "bufbufbuf.txt", "ram.txt", "rslatch.txt"];
    var examplesSelect = document.getElementById("exampleName");
    examples.forEach(function(e, i) {
        var option = document.createElement('option');
        option.selected = i === 0;
        option.value = e;
        option.text = e;
        examplesSelect.append(option);
    });
    dragElement(document.getElementById("draggableControls"));
});

function register_main_loop(main_loop) {
        var check_run_forever = document.getElementById("check_run_forever");
        var check_run_step = document.getElementById("check_run_step");
        var check_alive = document.getElementById("check_alive");
        var tick_display = document.getElementById("tick_display");
        var check_show_debug = document.getElementById("check_show_debug");
        var check_show_signals = document.getElementById("check_show_signals");
        var target_ticks_per_second = document.getElementById("target_ticks_per_second");
        var tick = 0;
        var intervalId;

        function demo() {
            if(check_run_forever.checked || check_run_step.checked) {
                main_loop(check_show_debug.checked, check_show_signals.checked);
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

        intervalId = setInterval(demo, 1000/30);
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

function runGui() {
    Rust.comphdl.then(function(comphdl) {
        var alive = document.getElementById("check_alive");
        // TODO: check if a thread is already running and kill it
        if (alive.checked) {
            // TODO: kill it
        }
        alive.checked = true;
        var error_string = comphdl.run_js_gui();
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
                            ig.style = "fill: #70FF70;";
                        } else {
                            ig.style = "fill: #147014;";
                        }
                    }
                };
                var ig = document.getElementById(inputid);
                if(ig) {
                    ig.style = "fill: #147014;";
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
