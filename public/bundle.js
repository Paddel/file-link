(function () {
    'use strict';

    let wasm;

    const heap = new Array(128).fill(undefined);

    heap.push(undefined, null, true, false);

    function getObject(idx) { return heap[idx]; }

    let heap_next = heap.length;

    function dropObject(idx) {
        if (idx < 132) return;
        heap[idx] = heap_next;
        heap_next = idx;
    }

    function takeObject(idx) {
        const ret = getObject(idx);
        dropObject(idx);
        return ret;
    }

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

    let WASM_VECTOR_LEN = 0;

    let cachedUint8Memory0 = null;

    function getUint8Memory0() {
        if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
            cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8Memory0;
    }

    const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

    const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
        ? function (arg, view) {
        return cachedTextEncoder.encodeInto(arg, view);
    }
        : function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    });

    function passStringToWasm0(arg, malloc, realloc) {

        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8Memory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }

        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
            const ret = encodeString(arg, view);

            offset += ret.written;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    let cachedInt32Memory0 = null;

    function getInt32Memory0() {
        if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
            cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
        }
        return cachedInt32Memory0;
    }

    const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

    if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); }
    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
    }

    let cachedFloat64Memory0 = null;

    function getFloat64Memory0() {
        if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
            cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
        }
        return cachedFloat64Memory0;
    }

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }

    function makeMutClosure(arg0, arg1, dtor, f) {
        const state = { a: arg0, b: arg1, cnt: 1, dtor };
        const real = (...args) => {
            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                if (--state.cnt === 0) {
                    wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

                } else {
                    state.a = a;
                }
            }
        };
        real.original = state;

        return real;
    }
    function __wbg_adapter_40(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h56b9fe6761044f29(arg0, arg1, addHeapObject(arg2));
    }

    let stack_pointer = 128;

    function addBorrowedObject(obj) {
        if (stack_pointer == 1) throw new Error('out of js stack');
        heap[--stack_pointer] = obj;
        return stack_pointer;
    }
    function __wbg_adapter_43(arg0, arg1, arg2) {
        try {
            wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd167725d6094d1b8(arg0, arg1, addBorrowedObject(arg2));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }

    function __wbg_adapter_48(arg0, arg1, arg2) {
        try {
            wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4c7b94c01c311806(arg0, arg1, addBorrowedObject(arg2));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }

    function __wbg_adapter_51(arg0, arg1, arg2) {
        wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hd96255f5c475369a(arg0, arg1, addHeapObject(arg2));
    }

    function __wbg_adapter_54(arg0, arg1, arg2) {
        try {
            wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h2ba4925c8cf28c34(arg0, arg1, addBorrowedObject(arg2));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }

    /**
    */
    function run_app() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.run_app(retptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            wasm.__wbindgen_exn_store(addHeapObject(e));
        }
    }

    let cachedUint32Memory0 = null;

    function getUint32Memory0() {
        if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
            cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
        }
        return cachedUint32Memory0;
    }

    function getArrayJsValueFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        const mem = getUint32Memory0();
        const slice = mem.subarray(ptr / 4, ptr / 4 + len);
        const result = [];
        for (let i = 0; i < slice.length; i++) {
            result.push(takeObject(slice[i]));
        }
        return result;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);

                } catch (e) {
                    if (module.headers.get('Content-Type') != 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else {
                        throw e;
                    }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);

        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };

            } else {
                return instance;
            }
        }
    }

    function __wbg_get_imports() {
        const imports = {};
        imports.wbg = {};
        imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
            takeObject(arg0);
        };
        imports.wbg.__wbindgen_cb_drop = function(arg0) {
            const obj = takeObject(arg0).original;
            if (obj.cnt-- == 1) {
                obj.a = 0;
                return true;
            }
            const ret = false;
            return ret;
        };
        imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        };
        imports.wbg.__wbg_crypto_c48a774b022d20ac = function(arg0) {
            const ret = getObject(arg0).crypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_is_object = function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        };
        imports.wbg.__wbg_process_298734cf255a885d = function(arg0) {
            const ret = getObject(arg0).process;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_versions_e2e78e134e3e5d01 = function(arg0) {
            const ret = getObject(arg0).versions;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_node_1cd7a5d853dbea79 = function(arg0) {
            const ret = getObject(arg0).node;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_is_string = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        };
        imports.wbg.__wbg_msCrypto_bcb970640f50a1e8 = function(arg0) {
            const ret = getObject(arg0).msCrypto;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_require_8f08ceecec0f4fee = function() { return handleError(function () {
            const ret = module.require;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbindgen_is_function = function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        };
        imports.wbg.__wbg_getRandomValues_37fa2ca9e4e07fab = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).getRandomValues(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_randomFillSync_dc1e9a60c158336d = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).randomFillSync(takeObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_listenerid_12315eee21527820 = function(arg0, arg1) {
            const ret = getObject(arg1).__yew_listener_id;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        };
        imports.wbg.__wbg_setlistenerid_3183aae8fa5840fb = function(arg0, arg1) {
            getObject(arg0).__yew_listener_id = arg1 >>> 0;
        };
        imports.wbg.__wbg_cachekey_b61393159c57fd7b = function(arg0, arg1) {
            const ret = getObject(arg1).__yew_subtree_cache_key;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        };
        imports.wbg.__wbg_subtreeid_e348577f7ef777e3 = function(arg0, arg1) {
            const ret = getObject(arg1).__yew_subtree_id;
            getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        };
        imports.wbg.__wbg_setsubtreeid_d32e6327eef1f7fc = function(arg0, arg1) {
            getObject(arg0).__yew_subtree_id = arg1 >>> 0;
        };
        imports.wbg.__wbg_setcachekey_80183b7cfc421143 = function(arg0, arg1) {
            getObject(arg0).__yew_subtree_cache_key = arg1 >>> 0;
        };
        imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
            const ret = new Error();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        };
        imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_is_undefined = function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        };
        imports.wbg.__wbindgen_in = function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_number_new = function(arg0) {
            const ret = arg0;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        };
        imports.wbg.__wbindgen_boolean_get = function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
            return ret;
        };
        imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
            getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
        };
        imports.wbg.__wbg_getwithrefkey_5e6d9547403deab8 = function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_set_841ac57cff3d672b = function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        };
        imports.wbg.__wbg_error_71d6845bf00a930f = function(arg0, arg1) {
            var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
            wasm.__wbindgen_free(arg0, arg1 * 4);
            console.error(...v0);
        };
        imports.wbg.__wbg_body_674aec4c1c0910cd = function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_createElement_4891554b28d3388b = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createElementNS_119acf9e82482041 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createTextNode_2fd22cd7e543f938 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_querySelector_52ded52c20e23921 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_instanceof_Window_9029196b662bc42a = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_document_f7ace2b956f30a4f = function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_location_56243dba507f472d = function(arg0) {
            const ret = getObject(arg0).location;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_history_3c2280e6b2a9316e = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).history;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_indexedDB_839701fb576f779c = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).indexedDB;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_alert_8c9ebcc791f5eaa8 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).alert(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_instanceof_IdbCursorWithValue_ce4b87159cd63444 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof IDBCursorWithValue;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_value_c49deffceeea3d79 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_deleteDatabase_c4a7d299bd51151e = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).deleteDatabase(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_open_6a08b03c958d4ad0 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).open(getStringFromWasm0(arg1, arg2), arg3 >>> 0);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_setonblocked_4311b52f166b3434 = function(arg0, arg1) {
            getObject(arg0).onblocked = getObject(arg1);
        };
        imports.wbg.__wbg_setonupgradeneeded_5a39a65558c323b2 = function(arg0, arg1) {
            getObject(arg0).onupgradeneeded = getObject(arg1);
        };
        imports.wbg.__wbg_types_fd4715a9278086c1 = function(arg0) {
            const ret = getObject(arg0).types;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_files_e5c28ff6ab126f7b = function(arg0) {
            const ret = getObject(arg0).files;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_contains_0f868d6343a00b39 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
            return ret;
        };
        imports.wbg.__wbg_candidate_5cef2e209cb0e642 = function(arg0, arg1) {
            const ret = getObject(arg1).candidate;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_sdpMid_76060b837f754e86 = function(arg0, arg1) {
            const ret = getObject(arg1).sdpMid;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_sdpMLineIndex_595a1839a9ccafb2 = function(arg0) {
            const ret = getObject(arg0).sdpMLineIndex;
            return isLikeNone(ret) ? 0xFFFFFF : ret;
        };
        imports.wbg.__wbg_new_fb5f24f7fbd85746 = function() { return handleError(function (arg0) {
            const ret = new RTCIceCandidate(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_target_f171e89c61e2bccf = function(arg0) {
            const ret = getObject(arg0).target;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_bubbles_63572b91f3885ef1 = function(arg0) {
            const ret = getObject(arg0).bubbles;
            return ret;
        };
        imports.wbg.__wbg_cancelBubble_90d1c3aa2a76cbeb = function(arg0) {
            const ret = getObject(arg0).cancelBubble;
            return ret;
        };
        imports.wbg.__wbg_composedPath_cf1bb5b8bcff496f = function(arg0) {
            const ret = getObject(arg0).composedPath();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_preventDefault_24104f3f0a54546a = function(arg0) {
            getObject(arg0).preventDefault();
        };
        imports.wbg.__wbg_parentNode_9e53f8b17eb98c9d = function(arg0) {
            const ret = getObject(arg0).parentNode;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_parentElement_c75962bc9997ea5f = function(arg0) {
            const ret = getObject(arg0).parentElement;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_lastChild_0cee692010bac6c2 = function(arg0) {
            const ret = getObject(arg0).lastChild;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_nextSibling_304d9aac7c2774ae = function(arg0) {
            const ret = getObject(arg0).nextSibling;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_setnodeValue_d1c8382910b45e04 = function(arg0, arg1, arg2) {
            getObject(arg0).nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_textContent_c5d9e21ee03c63d4 = function(arg0, arg1) {
            const ret = getObject(arg1).textContent;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_appendChild_51339d4cde00ee22 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_insertBefore_ffa01d4b747c95fc = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_removeChild_973429f368206138 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).removeChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_iceGatheringState_fea35c457cf86f05 = function(arg0) {
            const ret = getObject(arg0).iceGatheringState;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_iceConnectionState_929af9d5eb854451 = function(arg0) {
            const ret = getObject(arg0).iceConnectionState;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_setonicecandidate_ca0dbe4b8e912411 = function(arg0, arg1) {
            getObject(arg0).onicecandidate = getObject(arg1);
        };
        imports.wbg.__wbg_setoniceconnectionstatechange_242868ab1ffaf8ef = function(arg0, arg1) {
            getObject(arg0).oniceconnectionstatechange = getObject(arg1);
        };
        imports.wbg.__wbg_setonicegatheringstatechange_e0c5a4ab4d37ab63 = function(arg0, arg1) {
            getObject(arg0).onicegatheringstatechange = getObject(arg1);
        };
        imports.wbg.__wbg_setondatachannel_624a921c300dceba = function(arg0, arg1) {
            getObject(arg0).ondatachannel = getObject(arg1);
        };
        imports.wbg.__wbg_newwithconfiguration_c2620a61f13be424 = function() { return handleError(function (arg0) {
            const ret = new RTCPeerConnection(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_addIceCandidate_8108548506034a06 = function(arg0, arg1) {
            const ret = getObject(arg0).addIceCandidate(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createAnswer_ffe6dbcf7cd5ed2a = function(arg0) {
            const ret = getObject(arg0).createAnswer();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createDataChannel_69cc16b4f9cad344 = function(arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).createDataChannel(getStringFromWasm0(arg1, arg2), getObject(arg3));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_createOffer_aa7098f1f4c2f40b = function(arg0) {
            const ret = getObject(arg0).createOffer();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_setLocalDescription_4744eb2c267efbb4 = function(arg0, arg1) {
            const ret = getObject(arg0).setLocalDescription(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_setRemoteDescription_0c7a66e1bd51121d = function(arg0, arg1) {
            const ret = getObject(arg0).setRemoteDescription(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_dataTransfer_bac494821ce31837 = function(arg0) {
            const ret = getObject(arg0).dataTransfer;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_name_a46b2d975591a0b3 = function(arg0, arg1) {
            const ret = getObject(arg1).name;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_openCursor_555d508ba71b21cc = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).openCursor(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_put_fb32824d87feec5c = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).put(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_channel_33ffe429c66f76c2 = function(arg0) {
            const ret = getObject(arg0).channel;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_href_17ed54b321396524 = function(arg0, arg1) {
            const ret = getObject(arg1).href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_pathname_57290e07c6bc0683 = function(arg0, arg1) {
            const ret = getObject(arg1).pathname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_search_2ff3bb9114e0ca34 = function(arg0, arg1) {
            const ret = getObject(arg1).search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_hash_2b57e787945b2db0 = function(arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_sethash_41d6e65816639c62 = function(arg0, arg1, arg2) {
            getObject(arg0).hash = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_new_a76f6bcb38f791ea = function() { return handleError(function (arg0, arg1) {
            const ret = new URL(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_newwithbase_79b8cac27ce631ac = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = new URL(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_createObjectURL_d82f2880bada6a1d = function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_addEventListener_a5963e26cd7b176b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
        }, arguments) };
        imports.wbg.__wbg_removeEventListener_782040b4432709cb = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), arg4 !== 0);
        }, arguments) };
        imports.wbg.__wbg_instanceof_HtmlAnchorElement_a293f072b6174b83 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLAnchorElement;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_setdownload_0d874703cef6b180 = function(arg0, arg1, arg2) {
            getObject(arg0).download = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_sethref_a3fde9630423d8ed = function(arg0, arg1, arg2) {
            getObject(arg0).href = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_instanceof_IdbRequest_3a6ba410e62b59d8 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof IDBRequest;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_result_edff16ff107d6acb = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).result;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_setonsuccess_f518a37d8228a576 = function(arg0, arg1) {
            getObject(arg0).onsuccess = getObject(arg1);
        };
        imports.wbg.__wbg_setonerror_7bf21979c5219792 = function(arg0, arg1) {
            getObject(arg0).onerror = getObject(arg1);
        };
        imports.wbg.__wbg_href_d62a28e4fc1ab948 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_origin_50aa482fa6784a0a = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).origin;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_pathname_c8fd5c498079312d = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).pathname;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_search_6c3c472e076ee010 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_hash_a1a795b89dda8e3d = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg1).hash;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        }, arguments) };
        imports.wbg.__wbg_instanceof_Element_4622f5da1249a3eb = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Element;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_namespaceURI_31718ed49b5343a3 = function(arg0, arg1) {
            const ret = getObject(arg1).namespaceURI;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_setinnerHTML_b089587252408b67 = function(arg0, arg1, arg2) {
            getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_outerHTML_f7749ceff37b5832 = function(arg0, arg1) {
            const ret = getObject(arg1).outerHTML;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_children_27ed308801b57d3f = function(arg0) {
            const ret = getObject(arg0).children;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_removeAttribute_d8404da431968808 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_setAttribute_e7e80b478b7b8b2f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments) };
        imports.wbg.__wbg_instanceof_Blob_f2144a7f6c0c18a3 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Blob;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_size_b9bc39a333bd5d88 = function(arg0) {
            const ret = getObject(arg0).size;
            return ret;
        };
        imports.wbg.__wbg_newwithu8arraysequence_6951fb5a516ffb36 = function() { return handleError(function (arg0) {
            const ret = new Blob(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_arrayBuffer_27cefaea55cbf063 = function(arg0) {
            const ret = getObject(arg0).arrayBuffer();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_slice_5120ff009dfc5630 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).slice(arg1, arg2);
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_instanceof_ShadowRoot_b64337370f59fe2d = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ShadowRoot;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_host_e1c47c33975060d3 = function(arg0) {
            const ret = getObject(arg0).host;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_state_745dc4814d321eb3 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).state;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_pushState_1145414a47c0b629 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            getObject(arg0).pushState(getObject(arg1), getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
        }, arguments) };
        imports.wbg.__wbg_value_3c5f08ffc2b7d6f9 = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_setvalue_0dc100d4b9908028 = function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_only_9a8650679d219805 = function() { return handleError(function (arg0) {
            const ret = IDBKeyRange.only(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_readyState_b25418fd198bf715 = function(arg0) {
            const ret = getObject(arg0).readyState;
            return ret;
        };
        imports.wbg.__wbg_setbinaryType_096c70c4a9d97499 = function(arg0, arg1) {
            getObject(arg0).binaryType = takeObject(arg1);
        };
        imports.wbg.__wbg_new_b66404b6322c59bf = function() { return handleError(function (arg0, arg1) {
            const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_close_dfa389d8fddb52fc = function() { return handleError(function (arg0) {
            getObject(arg0).close();
        }, arguments) };
        imports.wbg.__wbg_send_280c8ab5d0df82de = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).send(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_href_47b90f0ddf3ddcd7 = function(arg0, arg1) {
            const ret = getObject(arg1).href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_instanceof_HtmlDocument_0485e23863c92f34 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof HTMLDocument;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_execCommand_eecccd1fb945375b = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).execCommand(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_instanceof_IdbDatabase_5d2aec935a1835ee = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof IDBDatabase;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_objectStoreNames_774e47f8818de433 = function(arg0) {
            const ret = getObject(arg0).objectStoreNames;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_close_29232080ee4839b2 = function(arg0) {
            getObject(arg0).close();
        };
        imports.wbg.__wbg_createObjectStore_4394a46abe4d85f5 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).createObjectStore(getStringFromWasm0(arg1, arg2), getObject(arg3));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_transaction_d6f1ef0b34b58a31 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).transaction(getStringFromWasm0(arg1, arg2), takeObject(arg3));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_objectStore_5a858a654147f96f = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).objectStore(getStringFromWasm0(arg1, arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_setchecked_e5a50baea447b8a8 = function(arg0, arg1) {
            getObject(arg0).checked = arg1 !== 0;
        };
        imports.wbg.__wbg_files_0b91078a034a0f7b = function(arg0) {
            const ret = getObject(arg0).files;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_value_9423da9d988ee8cf = function(arg0, arg1) {
            const ret = getObject(arg1).value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_setvalue_1f95e61cbc382f7f = function(arg0, arg1, arg2) {
            getObject(arg0).value = getStringFromWasm0(arg1, arg2);
        };
        imports.wbg.__wbg_valueAsNumber_c5b647430655b1f5 = function(arg0) {
            const ret = getObject(arg0).valueAsNumber;
            return ret;
        };
        imports.wbg.__wbg_select_8737496cf8f3fa7a = function(arg0) {
            getObject(arg0).select();
        };
        imports.wbg.__wbg_candidate_6839f7249e38bf91 = function(arg0) {
            const ret = getObject(arg0).candidate;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_click_9f2ce0ac84b1ce73 = function(arg0) {
            getObject(arg0).click();
        };
        imports.wbg.__wbg_length_b941879633a63ad8 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_item_77ac9b6a3db8c30a = function(arg0, arg1) {
            const ret = getObject(arg0).item(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        };
        imports.wbg.__wbg_instanceof_MessageEvent_395c9d7a213d29ed = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof MessageEvent;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_data_ab99ae4a2e1e8bc9 = function(arg0) {
            const ret = getObject(arg0).data;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_log_1d3ae0273d8f4f8a = function(arg0) {
            console.log(getObject(arg0));
        };
        imports.wbg.__wbg_result_58251a5d230b00f6 = function() { return handleError(function (arg0) {
            const ret = getObject(arg0).result;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_setonload_500a3ab3ebc0147b = function(arg0, arg1) {
            getObject(arg0).onload = getObject(arg1);
        };
        imports.wbg.__wbg_new_9b551002cd49569b = function() { return handleError(function () {
            const ret = new FileReader();
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_readAsArrayBuffer_07e155f1a3cd4ac2 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).readAsArrayBuffer(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_continue_b5a1b6eb1682ad84 = function() { return handleError(function (arg0) {
            getObject(arg0).continue();
        }, arguments) };
        imports.wbg.__wbg_label_58c11a5be3b0f8f1 = function(arg0, arg1) {
            const ret = getObject(arg1).label;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbg_readyState_60e14314ba4d3df9 = function(arg0) {
            const ret = getObject(arg0).readyState;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_setonopen_37846cc10560e3c0 = function(arg0, arg1) {
            getObject(arg0).onopen = getObject(arg1);
        };
        imports.wbg.__wbg_setonclose_d39802b4195bab2f = function(arg0, arg1) {
            getObject(arg0).onclose = getObject(arg1);
        };
        imports.wbg.__wbg_setonmessage_463c6aefedd50235 = function(arg0, arg1) {
            getObject(arg0).onmessage = getObject(arg1);
        };
        imports.wbg.__wbg_send_095f2fe0fd64b359 = function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).send(getStringFromWasm0(arg1, arg2));
        }, arguments) };
        imports.wbg.__wbg_send_1e6563347168b789 = function() { return handleError(function (arg0, arg1) {
            getObject(arg0).send(getObject(arg1));
        }, arguments) };
        imports.wbg.__wbg_get_44be0491f933a435 = function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_length_fff51ee6522a1a18 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_new_898a68150f225f2e = function() {
            const ret = new Array();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newnoargs_581967eacc0e2604 = function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_call_cb65541d95d71282 = function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_new_b51585de1b234aff = function() {
            const ret = new Object();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_self_1ff1d729e9aae938 = function() { return handleError(function () {
            const ret = self.self;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_window_5f4faef6c12b79ec = function() { return handleError(function () {
            const ret = window.window;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_globalThis_1d39714405582d3c = function() { return handleError(function () {
            const ret = globalThis.globalThis;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_global_651f05c6a0944d1c = function() { return handleError(function () {
            const ret = global.global;
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_at_f175831507f8dd5a = function(arg0, arg1) {
            const ret = getObject(arg0).at(arg1);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_from_d7c216d4616bb368 = function(arg0) {
            const ret = Array.from(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_push_ca1c26067ef907ac = function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_instanceof_ArrayBuffer_39ac22089b74fddb = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_byteLength_0488a7a303dccf40 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_toString_1c056108b87ba68b = function(arg0) {
            const ret = getObject(arg0).toString();
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_call_01734de55d61e11d = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_isSafeInteger_bb8e18dd21c97288 = function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        };
        imports.wbg.__wbg_entries_e51f29c7bba0c054 = function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_is_205d914af04a8faa = function(arg0, arg1) {
            const ret = Object.is(getObject(arg0), getObject(arg1));
            return ret;
        };
        imports.wbg.__wbg_resolve_53698b95aaf7fcf8 = function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_catch_64e0c7dcea0da34e = function(arg0, arg1) {
            const ret = getObject(arg0).catch(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_f7e06ee3c11698eb = function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_then_b2267541e2a73865 = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_buffer_085ec1f694018c4f = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_newwithbyteoffsetandlength_6da8e527659b86aa = function(arg0, arg1, arg2) {
            const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_new_8125e318e6245eed = function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_set_5cf90238115182c3 = function(arg0, arg1, arg2) {
            getObject(arg0).set(getObject(arg1), arg2 >>> 0);
        };
        imports.wbg.__wbg_length_72e2208bbc0efc61 = function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        };
        imports.wbg.__wbg_instanceof_Uint8Array_d8d9cb2b8e8ac1d4 = function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch {
                result = false;
            }
            const ret = result;
            return ret;
        };
        imports.wbg.__wbg_newwithlength_e5d69174d6984cd7 = function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_buffer_f5b7059c439f330d = function(arg0) {
            const ret = getObject(arg0).buffer;
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_subarray_13db269f57aa838d = function(arg0, arg1, arg2) {
            const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        };
        imports.wbg.__wbg_byteLength_47d11fa79875dee3 = function(arg0) {
            const ret = getObject(arg0).byteLength;
            return ret;
        };
        imports.wbg.__wbg_set_092e06b0f9d71865 = function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments) };
        imports.wbg.__wbg_parse_670c19d4e984792e = function() { return handleError(function (arg0, arg1) {
            const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbg_stringify_e25465938f3f611f = function() { return handleError(function (arg0) {
            const ret = JSON.stringify(getObject(arg0));
            return addHeapObject(ret);
        }, arguments) };
        imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getInt32Memory0()[arg0 / 4 + 1] = len1;
            getInt32Memory0()[arg0 / 4 + 0] = ptr1;
        };
        imports.wbg.__wbindgen_throw = function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        };
        imports.wbg.__wbindgen_memory = function() {
            const ret = wasm.memory;
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper359 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 56, __wbg_adapter_40);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper361 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 56, __wbg_adapter_43);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper363 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 56, __wbg_adapter_40);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper1845 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 865, __wbg_adapter_48);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper2168 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 994, __wbg_adapter_51);
            return addHeapObject(ret);
        };
        imports.wbg.__wbindgen_closure_wrapper2180 = function(arg0, arg1, arg2) {
            const ret = makeMutClosure(arg0, arg1, 998, __wbg_adapter_54);
            return addHeapObject(ret);
        };

        return imports;
    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedFloat64Memory0 = null;
        cachedInt32Memory0 = null;
        cachedUint32Memory0 = null;
        cachedUint8Memory0 = null;


        return wasm;
    }

    async function __wbg_init(input) {
        if (wasm !== undefined) return wasm;

        if (typeof input === 'undefined') {
            input = new URL('file_link_frontend_bg.wasm', (document.currentScript && document.currentScript.src || new URL('bundle.js', document.baseURI).href));
        }
        const imports = __wbg_get_imports();

        if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
            input = fetch(input);
        }

        const { instance, module } = await __wbg_load(await input, imports);

        return __wbg_finalize_init(instance, module);
    }

    async function main() {
       await __wbg_init('public/file_link_frontend_bg.wasm');
       run_app();
    }
    main();

})();
