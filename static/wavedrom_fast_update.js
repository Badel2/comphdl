function getWaveDromLane(lane_id) {
    return document.getElementById('wavelane_draw_' + lane_id + '_0');
}

function getLaneTickUse(lane, tick) {
    // TODO: The tick value at the start of the graph, default 0 but can
    // increase to simulate scrolling
    var initialTick = 0;
    var pos = tick - initialTick;
    return getLanePosUse(lane, pos);
}

function getLanePosUse(lane, pos) {
    return lane.childNodes[pos];
}

function setUseHref(b, value) {
    b.setAttribute('xlink:href', value);
}

function getUseHref(b) {
    return b.getAttribute('xlink:href');
}

function setWaveDromLaneTickValue(lane_id, pos, value) {
    // TODO: this will only set half tick
    var l = getWaveDromLane(lane_id);
    var b = getLanePosUse(l, pos);
    setUseHref(b, value);
}

function getWaveDromLaneTickValue(lane_id, pos) {
    var l = getWaveDromLane(lane_id);
    var b = getLanePosUse(l, pos);
    return getUseHref(b);
}

function shiftAllLanesLeftBy(n) {
    // WaveDrom uses half-ticks internally, we do not
    n = n * 2
    var i = 0;
    var l = getWaveDromLane(i);
    while (l) {
        var j = 0;
        var b = getLanePosUse(l, j);
        while (b) {
            var c = getLanePosUse(l, j + n);
            if (c) {
                setUseHref(b, getUseHref(c));
            } else {
                // The last n ticks are left untouched
            }
            j += 1;
            b = getLanePosUse(l, j);
        }
        i += 1;
        l = getWaveDromLane(i);
    }
}

function setPosAllLanes(pos, v) {
    // v is an array of "href": [ "#000", "#pclk", "#111", "#nclk" ]
    var i = 0;
    var l = getWaveDromLane(i);
    
    while (l) {
        var b0 = getLanePosUse(l, pos * 2);
        var b1 = getLanePosUse(l, pos * 2 + 1);
        if (v[i] == ".") {
            // "." means keep previous value
            // There may be glitches if we dont force b0 to b1
            setUseHref(b0, getUseHref(b1));
        } else if (v[i] == "h") {
            setUseHref(b0, "#pclk");
            setUseHref(b1, "#111");
        } else if (v[i] == "l") {
            setUseHref(b0, "#nclk");
            setUseHref(b1, "#000");
        } else if (v[i] == "X") {
            setUseHref(b0, "#xxx");
            setUseHref(b1, "#xxx");
        } else {
            console.error("Unsoported value: " + v[i]);
        }
        i += 1;
        l = getWaveDromLane(i);
    }
}

function setLastTickValues(vstr) {
    var lane0 = getWaveDromLane(0);
    if (!lane0) {
        return;
    }
    var num_ticks = lane0.children.length >> 1;
    if (num_ticks <= 0) {
        return;
    }
    shiftAllLanesLeftBy(1);
    var v = vstr.split(',');
    setPosAllLanes(num_ticks - 1, v);
}

var last_tick = 0;

export function refreshWaveDrom(WaveDrom, tick) {
    var v = document.getElementById('InputJSON_0_last');
    if (!v || tick == 0 || tick != last_tick + 1) {
        WaveDrom.editorRefresh();
    } else {
        // Assume we only need to shift by 1 tick, and add 1 new value to each lane
        // We could parse the WaveJson and extract the last values from there,
        // but we rely on external code to already place that values somewhere.
        setLastTickValues(v.value);
        // Hide tick markers, as they became incorrect now
        var g = document.getElementById('gmarks_0');
        for(var i = 0; i < g.children.length; i++) {
            var c = g.children[i];
            if(c.className.baseVal == "muted") {
                c.style.visibility = "hidden";
            }
        }
    }
    last_tick = tick;
    var t = document.getElementById('SVG');
    t.scrollLeft = t.scrollWidth;
}

