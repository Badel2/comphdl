parcelRequire=function(e,r,n,t){var i="function"==typeof parcelRequire&&parcelRequire,o="function"==typeof require&&require;function u(n,t){if(!r[n]){if(!e[n]){var f="function"==typeof parcelRequire&&parcelRequire;if(!t&&f)return f(n,!0);if(i)return i(n,!0);if(o&&"string"==typeof n)return o(n);var c=new Error("Cannot find module '"+n+"'");throw c.code="MODULE_NOT_FOUND",c}p.resolve=function(r){return e[n][1][r]||r},p.cache={};var l=r[n]=new u.Module(n);e[n][0].call(l.exports,p,l,l.exports,this)}return r[n].exports;function p(e){return u(p.resolve(e))}}u.isParcelRequire=!0,u.Module=function(e){this.id=e,this.bundle=u,this.exports={}},u.modules=e,u.cache=r,u.parent=i,u.register=function(r,n){e[r]=[function(e,r){r.exports=n},{}]};for(var f=0;f<n.length;f++)u(n[f]);if(n.length){var c=u(n[n.length-1]);"object"==typeof exports&&"undefined"!=typeof module?module.exports=c:"function"==typeof define&&define.amd?define(function(){return c}):t&&(this[t]=c)}return u}({"t6UB":[function(require,module,exports) {
var t=null;function r(){return t||(t=e()),t}function e(){try{throw new Error}catch(r){var t=(""+r.stack).match(/(https?|file|ftp):\/\/[^)\n]+/g);if(t)return n(t[0])}return"/"}function n(t){return(""+t).replace(/^((?:https?|file|ftp):\/\/.+)\/[^\/]+$/,"$1")+"/"}exports.getBundleURL=r,exports.getBaseURL=n;
},{}],"E0yX":[function(require,module,exports) {
var r=require("./bundle-url").getBundleURL;function e(r){Array.isArray(r)||(r=[r]);var e=r[r.length-1];try{return Promise.resolve(require(e))}catch(n){if("MODULE_NOT_FOUND"===n.code)return new s(function(n,i){t(r.slice(0,-1)).then(function(){return require(e)}).then(n,i)});throw n}}function t(r){return Promise.all(r.map(u))}var n={};function i(r,e){n[r]=e}module.exports=exports=e,exports.load=t,exports.register=i;var o={};function u(e){var t;if(Array.isArray(e)&&(t=e[1],e=e[0]),o[e])return o[e];var i=(e.substring(e.lastIndexOf(".")+1,e.length)||e).toLowerCase(),u=n[i];return u?o[e]=u(r()+e).then(function(r){return r&&module.bundle.register(t,r),r}).catch(function(r){throw delete o[e],r}):void 0}function s(r){this.executor=r,this.promise=null}s.prototype.then=function(r,e){return null===this.promise&&(this.promise=new Promise(this.executor)),this.promise.then(r,e)},s.prototype.catch=function(r){return null===this.promise&&(this.promise=new Promise(this.executor)),this.promise.catch(r)};
},{"./bundle-url":"t6UB"}],"c5+M":[function(require,module,exports) {
function e(r){return(e="function"==typeof Symbol&&"symbol"==typeof Symbol.iterator?function(e){return typeof e}:function(e){return e&&"function"==typeof Symbol&&e.constructor===Symbol&&e!==Symbol.prototype?"symbol":typeof e})(r)}module.exports=function(r){function t(r,t){return _=function(){var r={STDWEB_PRIVATE:{}};r.STDWEB_PRIVATE.to_utf8=function(e,r){for(var t=0;t<e.length;++t){var _=e.charCodeAt(t);_>=55296&&_<=57343&&(_=65536+((1023&_)<<10)|1023&e.charCodeAt(++t)),_<=127?c[r++]=_:_<=2047?(c[r++]=192|_>>6,c[r++]=128|63&_):_<=65535?(c[r++]=224|_>>12,c[r++]=128|_>>6&63,c[r++]=128|63&_):_<=2097151?(c[r++]=240|_>>18,c[r++]=128|_>>12&63,c[r++]=128|_>>6&63,c[r++]=128|63&_):_<=67108863?(c[r++]=248|_>>24,c[r++]=128|_>>18&63,c[r++]=128|_>>12&63,c[r++]=128|_>>6&63,c[r++]=128|63&_):(c[r++]=252|_>>30,c[r++]=128|_>>24&63,c[r++]=128|_>>18&63,c[r++]=128|_>>12&63,c[r++]=128|_>>6&63,c[r++]=128|63&_)}},r.STDWEB_PRIVATE.noop=function(){},r.STDWEB_PRIVATE.to_js=function(e){var t=c[e+12];if(0!==t){if(1===t)return null;if(2===t)return a[e/4];if(3===t)return E[e/8];if(4===t){var _=u[e/4],T=u[(e+4)/4];return r.STDWEB_PRIVATE.to_js_string(_,T)}if(5===t)return!1;if(6===t)return!0;if(7===t){for(var _=r.STDWEB_PRIVATE.arena+u[e/4],T=u[(e+4)/4],l=[],s=0;s<T;++s)l.push(r.STDWEB_PRIVATE.to_js(_+16*s));return l}if(8===t){for(var d=r.STDWEB_PRIVATE.arena,A=d+u[e/4],T=u[(e+4)/4],b=d+u[(e+8)/4],l={},s=0;s<T;++s){var p=u[(b+8*s)/4],I=u[(b+4+8*s)/4],S=r.STDWEB_PRIVATE.to_js_string(p,I),B=r.STDWEB_PRIVATE.to_js(A+16*s);l[S]=B}return l}if(9===t)return r.STDWEB_PRIVATE.acquire_js_reference(a[e/4]);if(10===t||12===t||13===t){var R=u[e/4],_=u[(e+4)/4],D=u[(e+8)/4],P=0,W=!1,l=function(){if(0===_||!0===W)throw 10===t?new ReferenceError("Already dropped Rust function called!"):12===t?new ReferenceError("Already dropped FnMut function called!"):new ReferenceError("Already called or dropped FnOnce function called!");var e=_;if(13===t&&(l.drop=r.STDWEB_PRIVATE.noop,_=0),0!==P&&(12===t||13===t))throw new ReferenceError("FnMut function called multiple times concurrently!");var n=r.STDWEB_PRIVATE.alloc(16);r.STDWEB_PRIVATE.serialize_array(n,arguments);try{P+=1,r.STDWEB_PRIVATE.dyncall("vii",R,[e,n]);var o=r.STDWEB_PRIVATE.tmp;r.STDWEB_PRIVATE.tmp=null}finally{P-=1}return!0===W&&0===P&&l.drop(),o};return l.drop=function(){if(0===P){l.drop=r.STDWEB_PRIVATE.noop;var e=_;_=0,0!=e&&r.STDWEB_PRIVATE.dyncall("vi",D,[e])}else W=!0},l}if(14===t){var _=u[e/4],T=u[(e+4)/4],V=u[(e+8)/4],m=_+T;switch(V){case 0:return c.subarray(_,m);case 1:return n.subarray(_,m);case 2:return i.subarray(_,m);case 3:return o.subarray(_,m);case 4:return u.subarray(_,m);case 5:return a.subarray(_,m);case 6:return f.subarray(_,m);case 7:return E.subarray(_,m)}}else if(15===t)return r.STDWEB_PRIVATE.get_raw_value(u[e/4])}},r.STDWEB_PRIVATE.serialize_object=function(e,t){var _=Object.keys(t),n=_.length,o=r.STDWEB_PRIVATE.alloc(8*n),a=r.STDWEB_PRIVATE.alloc(16*n);c[e+12]=8,u[e/4]=a,u[(e+4)/4]=n,u[(e+8)/4]=o;for(var i=0;i<n;++i){var f=_[i],E=o+8*i;r.STDWEB_PRIVATE.to_utf8_string(E,f),r.STDWEB_PRIVATE.from_js(a+16*i,t[f])}},r.STDWEB_PRIVATE.serialize_array=function(e,t){var _=t.length,n=r.STDWEB_PRIVATE.alloc(16*_);c[e+12]=7,u[e/4]=n,u[(e+4)/4]=_;for(var o=0;o<_;++o)r.STDWEB_PRIVATE.from_js(n+16*o,t[o])};var t="function"==typeof TextEncoder?new TextEncoder("utf-8"):"object"===("undefined"==typeof util?"undefined":e(util))&&util&&"function"==typeof util.TextEncoder?new util.TextEncoder("utf-8"):null;r.STDWEB_PRIVATE.to_utf8_string=null!=t?function(e,_){var n=t.encode(_),o=n.length,a=0;o>0&&(a=r.STDWEB_PRIVATE.alloc(o),c.set(n,a)),u[e/4]=a,u[(e+4)/4]=o}:function(e,t){var _=r.STDWEB_PRIVATE.utf8_len(t),n=0;_>0&&(n=r.STDWEB_PRIVATE.alloc(_),r.STDWEB_PRIVATE.to_utf8(t,n)),u[e/4]=n,u[(e+4)/4]=_};r.STDWEB_PRIVATE.from_js=function(e,t){var _=Object.prototype.toString.call(t);if("[object String]"===_)c[e+12]=4,r.STDWEB_PRIVATE.to_utf8_string(e,t);else if("[object Number]"===_)t===(0|t)?(c[e+12]=2,a[e/4]=t):(c[e+12]=3,E[e/8]=t);else if(null===t)c[e+12]=1;else if(void 0===t)c[e+12]=0;else if(!1===t)c[e+12]=5;else if(!0===t)c[e+12]=6;else if("[object Symbol]"===_){var n=r.STDWEB_PRIVATE.register_raw_value(t);c[e+12]=15,a[e/4]=n}else{var o=r.STDWEB_PRIVATE.acquire_rust_reference(t);c[e+12]=9,a[e/4]=o}};var _="function"==typeof TextDecoder?new TextDecoder("utf-8"):"object"===("undefined"==typeof util?"undefined":e(util))&&util&&"function"==typeof util.TextDecoder?new util.TextDecoder("utf-8"):null;r.STDWEB_PRIVATE.to_js_string=null!=_?function(e,r){return _.decode(c.subarray(e,e+r))}:function(e,r){for(var t=(0|(e|=0))+(0|(r|=0)),_="";e<t;){var n=c[e++];if(n<128)_+=String.fromCharCode(n);else{var o=31&n,a=0;e<t&&(a=c[e++]);var i=o<<6|63&a;if(n>=224){var u=0;e<t&&(u=c[e++]);var f=(63&a)<<6|63&u;if(i=o<<12|f,n>=240){var E=0;e<t&&(E=c[e++]),i=(7&o)<<18|f<<6|63&E,_+=String.fromCharCode(55232+(i>>10)),i=56320+(1023&i)}}_+=String.fromCharCode(i)}}return _};r.STDWEB_PRIVATE.id_to_ref_map={},r.STDWEB_PRIVATE.id_to_refcount_map={},r.STDWEB_PRIVATE.ref_to_id_map=new WeakMap,r.STDWEB_PRIVATE.ref_to_id_map_fallback=new Map,r.STDWEB_PRIVATE.last_refid=1,r.STDWEB_PRIVATE.id_to_raw_value_map={},r.STDWEB_PRIVATE.last_raw_value_id=1,r.STDWEB_PRIVATE.acquire_rust_reference=function(e){if(null==e)return 0;var t=r.STDWEB_PRIVATE.id_to_refcount_map,_=r.STDWEB_PRIVATE.id_to_ref_map,n=r.STDWEB_PRIVATE.ref_to_id_map,o=r.STDWEB_PRIVATE.ref_to_id_map_fallback,a=n.get(e);if(void 0===a&&(a=o.get(e)),void 0===a){a=r.STDWEB_PRIVATE.last_refid++;try{n.set(e,a)}catch(c){o.set(e,a)}}return a in _?t[a]++:(_[a]=e,t[a]=1),a},r.STDWEB_PRIVATE.acquire_js_reference=function(e){return r.STDWEB_PRIVATE.id_to_ref_map[e]},r.STDWEB_PRIVATE.increment_refcount=function(e){r.STDWEB_PRIVATE.id_to_refcount_map[e]++},r.STDWEB_PRIVATE.decrement_refcount=function(e){var t=r.STDWEB_PRIVATE.id_to_refcount_map;if(0==--t[e]){var _=r.STDWEB_PRIVATE.id_to_ref_map,n=r.STDWEB_PRIVATE.ref_to_id_map_fallback,o=_[e];delete _[e],delete t[e],n.delete(o)}},r.STDWEB_PRIVATE.register_raw_value=function(e){var t=r.STDWEB_PRIVATE.last_raw_value_id++;return r.STDWEB_PRIVATE.id_to_raw_value_map[t]=e,t},r.STDWEB_PRIVATE.unregister_raw_value=function(e){delete r.STDWEB_PRIVATE.id_to_raw_value_map[e]},r.STDWEB_PRIVATE.get_raw_value=function(e){return r.STDWEB_PRIVATE.id_to_raw_value_map[e]},r.STDWEB_PRIVATE.alloc=function(e){return r.web_malloc(e)},r.STDWEB_PRIVATE.dyncall=function(e,t,_){return r.web_table.get(t).apply(null,_)},r.STDWEB_PRIVATE.utf8_len=function(e){for(var r=0,t=0;t<e.length;++t){var _=e.charCodeAt(t);_>=55296&&_<=57343&&(_=65536+((1023&_)<<10)|1023&e.charCodeAt(++t)),_<=127?++r:r+=_<=2047?2:_<=65535?3:_<=2097151?4:_<=67108863?5:6}return r},r.STDWEB_PRIVATE.prepare_any_arg=function(e){var t=r.STDWEB_PRIVATE.alloc(16);return r.STDWEB_PRIVATE.from_js(t,e),t},r.STDWEB_PRIVATE.acquire_tmp=function(e){var t=r.STDWEB_PRIVATE.tmp;return r.STDWEB_PRIVATE.tmp=null,t};var n=null,o=null,a=null,c=null,i=null,u=null,f=null,E=null;function T(){var e=r.instance.exports.memory.buffer;n=new Int8Array(e),o=new Int16Array(e),a=new Int32Array(e),c=new Uint8Array(e),i=new Uint16Array(e),u=new Uint32Array(e),f=new Float32Array(e),E=new Float64Array(e)}return Object.defineProperty(r,"exports",{value:{}}),{imports:{env:{__cargo_web_snippet_05e76530774cc7d3b1b5e804905b95575736d825:function(e,t,_){var n;t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,void((n=document.getElementById("checkbox_output_i"+t))&&(n.checked=_)))},__cargo_web_snippet_0b84387678fcb2ace6bd4fa4e0edf6e0cf20f57b:function(e,t,_){var n;t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,void((n=document.getElementById("outputExt_"+t))&&(n.style="fill: "+_)))},__cargo_web_snippet_25a53fc5779c4b9e2e13a4f7713062ff17cab110:function(e,t,_){t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,(document.getElementById("InputJSON_0").value=t,void(document.getElementById("InputJSON_0_last").value=_)))},__cargo_web_snippet_352943ae98b2eeb817e36305c3531d61c7e1a52b:function(e){return r.STDWEB_PRIVATE.acquire_js_reference(e)instanceof Element|0},__cargo_web_snippet_358b76356a247f58406de0646edb9c92d0531794:function(e,t,_){t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,(document.getElementById("top_input").innerHTML=t,void(document.getElementById("top_output").innerHTML=_)))},__cargo_web_snippet_4fd31c9e56d40b8642cf9e6f96fd6b570f355cea:function(e){e=r.STDWEB_PRIVATE.to_js(e),console.error(e)},__cargo_web_snippet_614a3dd2adb7e9eac4a0ec6e59d37f87e0521c3b:function(e,t){t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,t.error)},__cargo_web_snippet_6ae67955866a020b84db93ce246a401aa5096d7f:function(e,t,_,n){t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),n=r.STDWEB_PRIVATE.to_js(n),r.STDWEB_PRIVATE.from_js(e,function(){for(var e=document.querySelectorAll(t),r=0;r<e.length;r++)e[r].style.stroke=_,e[r].style.strokeWidth=n}())},__cargo_web_snippet_6b751c2f029ce9350927f80c6587c8f66589705e:function(e,t){var _;t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,null==(_=document.getElementById(t))?null:_.value)},__cargo_web_snippet_6fcce0aae651e2d748e085ff1f800f87625ff8c8:function(e){r.STDWEB_PRIVATE.from_js(e,document)},__cargo_web_snippet_7c8dfab835dc8a552cd9d67f27d26624590e052c:function(e){var t=r.STDWEB_PRIVATE.acquire_js_reference(e);return t instanceof DOMException&&"SyntaxError"===t.name},__cargo_web_snippet_80d6d56760c65e49b7be8b6b01c1ea861b046bf0:function(e){r.STDWEB_PRIVATE.decrement_refcount(e)},__cargo_web_snippet_8545f3ba2883a49a2afd23c48c5d24ef3f9b0071:function(e){return r.STDWEB_PRIVATE.acquire_js_reference(e)instanceof HTMLTextAreaElement|0},__cargo_web_snippet_909a2fecd4c4be79a93f372d8e59cb43d4840227:function(e,t){t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,void require("_bundle_loader")(require.resolve("/index.js")).then(function(e){e.term.write(t)}))},__cargo_web_snippet_a152e8d0e8fac5476f30c1d19e4ab217dbcba73d:function(e,t,_){t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,function(){try{return{value:t.querySelector(_),success:!0}}catch(e){return{error:e,success:!1}}}())},__cargo_web_snippet_a444f6bffad0a44e632a33b3e4d617a33fd4ece2:function(e,t,_){var n;t=r.STDWEB_PRIVATE.to_js(t),_=r.STDWEB_PRIVATE.to_js(_),r.STDWEB_PRIVATE.from_js(e,void(null!=(n=document.getElementById(t))&&(n.value+=_,n.scrollTop=n.scrollHeight)))},__cargo_web_snippet_ab05f53189dacccf2d365ad26daa407d4f7abea9:function(e,t){t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,t.value)},__cargo_web_snippet_b06dde4acf09433b5190a4b001259fe5d4abcbc2:function(e,t){t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,t.success)},__cargo_web_snippet_bc7e6c64bc1e3d3cc1f40302b950f939791c39d8:function(e,t){var _;t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,(null!=(_=document.getElementById(t))&&(_.value=""),void require("_bundle_loader")(require.resolve("/index.js")).then(function(e){e.term.clear(),e.term2.clear()})))},__cargo_web_snippet_c1e01b18532cb0bcaddbb94f6ce72a714d01bd75:function(e,t){t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,void require("_bundle_loader")(require.resolve("/index.js")).then(function(e){e.term2.writeln(t)}))},__cargo_web_snippet_c26ddf75f581148e029dfcd95c037bb50d502e43:function(e,t){e=r.STDWEB_PRIVATE.to_js(e),t=r.STDWEB_PRIVATE.to_js(t),e.value=t},__cargo_web_snippet_cb4cd3d40f96b12d3dedb5aee0104128d2c0f5ac:function(e,t){var _;t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,(_=t,void require("_bundle_loader")(require.resolve("/index.js")).then(function(e){e.register_main_loop(_)})))},__cargo_web_snippet_dc81b01621f8a04ff579d34ee57fcc66c8cbd7e8:function(e,t){var _;t=r.STDWEB_PRIVATE.to_js(t),r.STDWEB_PRIVATE.from_js(e,null==(_=document.getElementById("checkbox_input_"+t))?null:_.checked)},__cargo_web_snippet_e9638d6405ab65f78daf4a5af9c9de14ecf1e2ec:function(e){e=r.STDWEB_PRIVATE.to_js(e),r.STDWEB_PRIVATE.unregister_raw_value(e)},__cargo_web_snippet_ff5103e6cc179d13b4c7a785bdce2708fd559fc0:function(e){r.STDWEB_PRIVATE.tmp=r.STDWEB_PRIVATE.to_js(e)},__web_on_grow:T}},initialize:function(e){return Object.defineProperty(r,"instance",{value:e}),Object.defineProperty(r,"web_malloc",{value:r.instance.exports.__web_malloc}),Object.defineProperty(r,"web_free",{value:r.instance.exports.__web_free}),Object.defineProperty(r,"web_table",{value:r.instance.exports.__indirect_function_table}),r.exports.run_js_gui=function(e){return r.STDWEB_PRIVATE.acquire_tmp(r.instance.exports.run_js_gui(r.STDWEB_PRIVATE.prepare_any_arg(e)))},T(),r.instance.exports.main(),r.exports}}}(),t?WebAssembly.instantiate(r,_.imports).then(function(e){var r=_.initialize(e);return console.log("Finished loading Rust wasm module 'comphdl_web'"),r}).catch(function(e){throw console.log("Error loading Rust wasm module 'comphdl_web':",e),e}):(_=new WebAssembly.Instance(r,_.imports)).initialize(wasm_instance);var _}return fetch(r).then(function(e){return e.arrayBuffer()}).then(function(e){return WebAssembly.compile(e)}).then(function(e){return t(e,!0)})};
},{"_bundle_loader":"E0yX","/index.js":[["static.5b0e70c2.js","Focm"],"static.699d5e06.map",["Cargo.1f613bf6.cargo-web-398fc556ba2b08aea6f4fadc3880dfdf","ubyA"],["comphdl.829039e7.svg","dU0r"],"Focm"]}],"V7it":[function(require,module,exports) {
module.exports=function(e){return require("./loader-398fc556ba2b08aea6f4fadc3880dfdf.js")(e)};
},{"./loader-398fc556ba2b08aea6f4fadc3880dfdf.js":"c5+M"}],"/bzd":[function(require,module,exports) {
module.exports=function(n){return new Promise(function(e,o){var r=document.createElement("script");r.async=!0,r.type="text/javascript",r.charset="utf-8",r.src=n,r.onerror=function(n){r.onerror=r.onload=null,o(n)},r.onload=function(){r.onerror=r.onload=null,e()},document.getElementsByTagName("head")[0].appendChild(r)})};
},{}],0:[function(require,module,exports) {
var b=require("E0yX");b.register("cargo-web-398fc556ba2b08aea6f4fadc3880dfdf",require("V7it"));b.register("js",require("/bzd"));
},{}]},{},[0], null)