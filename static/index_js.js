Rust.comphdl.then(function(comphdl) {
    document.getElementById("loading_wasm").style.display = "none";
    document.getElementById("loaded_wasm").style.display = "block";

    loadExample(false, "example1.txt");
    var examples = ["example1.txt", "bufbufbuf.txt", "rslatch.txt"];
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

function loadExampleSelect() {
    var name = document.getElementById("exampleName").value;
    stopSimulation();
    loadExample(true, name);
}

function loadExample(force, name) {
    var a = document.getElementById("comphdl_definition");
    // Do not overwrite existing code
    if (force == false && a.value != "") {
        return;
    }
    fetch("./comphdl_examples/" + name).then(function(response) {
        if (!response.ok) {
            throw Error(response.statusText);
        }
        return response;
    }).then(function(response) {
        response.text().then(function(text) {
            a.value = text;
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
        comphdl.run_js_gui();
        showSimulationControls();
        document.getElementById("renderButton").click();
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