
let wasm;

/**
*/
export function main() {
    wasm.main();
}

const heap = new Array(32);

heap.fill(undefined);

heap.push(undefined, null, true, false);

let stack_pointer = 32;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

let passStringToWasm;
if (typeof cachedTextEncoder.encodeInto === 'function') {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            arg = arg.slice(offset);
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + arg.length * 3);
            const view = getUint8Memory().subarray(ptr + offset, ptr + size);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
} else {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            const buf = cachedTextEncoder.encode(arg.slice(offset));
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + buf.length);
            getUint8Memory().set(buf, ptr + offset);
            offset += buf.length;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
}

let cachegetUint32Memory = null;
function getUint32Memory() {
    if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function passArrayJsValueToWasm(array) {
    const ptr = wasm.__wbindgen_malloc(array.length * 4);
    const mem = getUint32Memory();
    for (let i = 0; i < array.length; i++) {
        mem[ptr / 4 + i] = addHeapObject(array[i]);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

let cachegetInt32Memory = null;
function getInt32Memory() {
    if (cachegetInt32Memory === null || cachegetInt32Memory.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory;
}

function handleError(e) {
    wasm.__wbindgen_exn_store(addHeapObject(e));
}
/**
*/
export class LincolnIntepretor {

    static __wrap(ptr) {
        const obj = Object.create(LincolnIntepretor.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_lincolnintepretor_free(ptr);
    }
    /**
    * @returns {LincolnIntepretor}
    */
    static new() {
        const ret = wasm.lincolnintepretor_new();
        return LincolnIntepretor.__wrap(ret);
    }
    /**
    * @param {any} prog
    */
    set_program(prog) {
        try {
            wasm.lincolnintepretor_set_program(this.ptr, addBorrowedObject(prog));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @returns {any}
    */
    get_program() {
        const ret = wasm.lincolnintepretor_get_program(this.ptr);
        return takeObject(ret);
    }
    /**
    * @param {string} jmplabel
    * @param {string} jmpcont
    * @param {number} per
    */
    jmp(jmplabel, jmpcont, per) {
        wasm.lincolnintepretor_jmp(this.ptr, passStringToWasm(jmplabel), WASM_VECTOR_LEN, passStringToWasm(jmpcont), WASM_VECTOR_LEN, per);
    }
    /**
    * @param {string} calllabel
    * @param {string} callee
    * @param {number} callcnt
    * @param {string} callcont
    */
    call(calllabel, callee, callcnt, callcont) {
        wasm.lincolnintepretor_call(this.ptr, passStringToWasm(calllabel), WASM_VECTOR_LEN, passStringToWasm(callee), WASM_VECTOR_LEN, callcnt, passStringToWasm(callcont), WASM_VECTOR_LEN);
    }
    /**
    * @param {string} retlabel
    * @param {number} variant
    */
    ret(retlabel, variant) {
        wasm.lincolnintepretor_ret(this.ptr, passStringToWasm(retlabel), WASM_VECTOR_LEN, variant);
    }
    /**
    * @param {string} grouplabel
    * @param {any[]} elements
    */
    group(grouplabel, elements) {
        wasm.lincolnintepretor_group(this.ptr, passStringToWasm(grouplabel), WASM_VECTOR_LEN, passArrayJsValueToWasm(elements), WASM_VECTOR_LEN);
    }
    /**
    * @param {string} exportlabel
    */
    set_export(exportlabel) {
        wasm.lincolnintepretor_set_export(this.ptr, passStringToWasm(exportlabel), WASM_VECTOR_LEN);
    }
    /**
    * @param {string} deletelabel
    */
    delete(deletelabel) {
        wasm.lincolnintepretor_delete(this.ptr, passStringToWasm(deletelabel), WASM_VECTOR_LEN);
    }
    /**
    * @param {any} externs
    */
    compile(externs) {
        try {
            wasm.lincolnintepretor_compile(this.ptr, addBorrowedObject(externs));
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @param {string} entry
    * @param {number} variant
    * @param {any} values
    * @param {boolean} step
    */
    run(entry, variant, values, step) {
        try {
            wasm.lincolnintepretor_run(this.ptr, passStringToWasm(entry), WASM_VECTOR_LEN, variant, addBorrowedObject(values), step);
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @returns {boolean}
    */
    step() {
        const ret = wasm.lincolnintepretor_step(this.ptr);
        return ret !== 0;
    }
    /**
    * @returns {any}
    */
    get_context() {
        const ret = wasm.lincolnintepretor_get_context(this.ptr);
        return takeObject(ret);
    }
}
/**
*/
export class LincolnJsValue {

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_lincolnjsvalue_free(ptr);
    }
}

function init(module) {
    if (typeof module === 'undefined') {
        module = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    let result;
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_json_parse = function(arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_json_serialize = function(arg0, arg1) {
        const ret = JSON.stringify(getObject(arg1));
        const ret0 = passStringToWasm(ret);
        const ret1 = WASM_VECTOR_LEN;
        getInt32Memory()[arg0 / 4 + 0] = ret0;
        getInt32Memory()[arg0 / 4 + 1] = ret1;
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'function';
        return ret;
    };
    imports.wbg.__widl_f_debug_1_ = function(arg0) {
        console.debug(getObject(arg0));
    };
    imports.wbg.__widl_f_error_1_ = function(arg0) {
        console.error(getObject(arg0));
    };
    imports.wbg.__widl_f_info_1_ = function(arg0) {
        console.info(getObject(arg0));
    };
    imports.wbg.__widl_f_log_1_ = function(arg0) {
        console.log(getObject(arg0));
    };
    imports.wbg.__widl_f_warn_1_ = function(arg0) {
        console.warn(getObject(arg0));
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbg_next_e3e7fc4e13e40e55 = function(arg0) {
        const ret = getObject(arg0).next;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_next_17e79f725ff5a87a = function(arg0) {
        try {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_done_d485bad1edfcebc6 = function(arg0) {
        const ret = getObject(arg0).done;
        return ret;
    };
    imports.wbg.__wbg_value_ce1d7ad603d82534 = function(arg0) {
        const ret = getObject(arg0).value;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_iterator_6974146c73f389ba = function() {
        const ret = Symbol.iterator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_get_f54fa02552389dda = function(arg0, arg1) {
        try {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_call_1fc553129cb17c3c = function(arg0, arg1) {
        try {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_new_f1f0f3113e466334 = function() {
        const ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_unshift_b38ba295bba3cfd7 = function(arg0, arg1) {
        const ret = getObject(arg0).unshift(getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_apply_a7d91e1867ff2ba0 = function(arg0, arg1, arg2) {
        try {
            const ret = getObject(arg0).apply(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        } catch (e) {
            handleError(e)
        }
    };
    imports.wbg.__wbg_name_146e7e39df264e29 = function(arg0) {
        const ret = getObject(arg0).name;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg0);
        if (typeof(obj) !== 'string') return 0;
        const ptr = passStringToWasm(obj);
        getUint32Memory()[arg1 / 4] = WASM_VECTOR_LEN;
        const ret = ptr;
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm(arg0, arg1));
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw takeObject(arg0);
    };

    if (module instanceof URL || typeof module === 'string' || module instanceof Request) {

        const response = fetch(module);
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            result = WebAssembly.instantiateStreaming(response, imports)
            .catch(e => {
                console.warn("`WebAssembly.instantiateStreaming` failed. Assuming this is because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                return response
                .then(r => r.arrayBuffer())
                .then(bytes => WebAssembly.instantiate(bytes, imports));
            });
        } else {
            result = response
            .then(r => r.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, imports));
        }
    } else {

        result = WebAssembly.instantiate(module, imports)
        .then(result => {
            if (result instanceof WebAssembly.Instance) {
                return { instance: result, module };
            } else {
                return result;
            }
        });
    }
    return result.then(({instance, module}) => {
        wasm = instance.exports;
        init.__wbindgen_wasm_module = module;
        wasm.__wbindgen_start();
        return wasm;
    });
}

export default init;

