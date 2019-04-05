const __exports = {};

let wasm;

/**
* @returns {void}
*/
export function main() {
    return wasm.main();
}

__exports.main = main;

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

let cachedTextEncoder = new TextEncoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

let WASM_VECTOR_LEN = 0;

let passStringToWasm;
if (typeof cachedTextEncoder.encodeInto === 'function') {
    passStringToWasm = function(arg) {

        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let writeOffset = 0;
        while (true) {
            const view = getUint8Memory().subarray(ptr + writeOffset, ptr + size);
            const { read, written } = cachedTextEncoder.encodeInto(arg, view);
            arg = arg.substring(read);
            writeOffset += written;
            if (arg.length === 0) {
                break;
            }
            ptr = wasm.__wbindgen_realloc(ptr, size, size * 2);
            size *= 2;
        }
        WASM_VECTOR_LEN = writeOffset;
        return ptr;
    };
} else {
    passStringToWasm = function(arg) {

        const buf = cachedTextEncoder.encode(arg);
        const ptr = wasm.__wbindgen_malloc(buf.length);
        getUint8Memory().set(buf, ptr);
        WASM_VECTOR_LEN = buf.length;
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

function __widl_f_debug_1_(arg0) {
    console.debug(getObject(arg0));
}

__exports.__widl_f_debug_1_ = __widl_f_debug_1_;

function __widl_f_error_1_(arg0) {
    console.error(getObject(arg0));
}

__exports.__widl_f_error_1_ = __widl_f_error_1_;

function __widl_f_info_1_(arg0) {
    console.info(getObject(arg0));
}

__exports.__widl_f_info_1_ = __widl_f_info_1_;

function __widl_f_log_1_(arg0) {
    console.log(getObject(arg0));
}

__exports.__widl_f_log_1_ = __widl_f_log_1_;

function __widl_f_warn_1_(arg0) {
    console.warn(getObject(arg0));
}

__exports.__widl_f_warn_1_ = __widl_f_warn_1_;

function __wbg_new_816c11756f2e51ab() {
    return addHeapObject(new Array());
}

__exports.__wbg_new_816c11756f2e51ab = __wbg_new_816c11756f2e51ab;

function __wbg_unshift_602f54fc3a5021cc(arg0, arg1) {
    return getObject(arg0).unshift(getObject(arg1));
}

__exports.__wbg_unshift_602f54fc3a5021cc = __wbg_unshift_602f54fc3a5021cc;

function handleError(exnptr, e) {
    const view = getUint32Memory();
    view[exnptr / 4] = 1;
    view[exnptr / 4 + 1] = addHeapObject(e);
}

function __wbg_apply_ccde16c927c5f8be(arg0, arg1, arg2, exnptr) {
    try {
        return addHeapObject(getObject(arg0).apply(getObject(arg1), getObject(arg2)));
    } catch (e) {
        handleError(exnptr, e);
    }
}

__exports.__wbg_apply_ccde16c927c5f8be = __wbg_apply_ccde16c927c5f8be;

function __wbg_call_a7a8823c404228ab(arg0, arg1, exnptr) {
    try {
        return addHeapObject(getObject(arg0).call(getObject(arg1)));
    } catch (e) {
        handleError(exnptr, e);
    }
}

__exports.__wbg_call_a7a8823c404228ab = __wbg_call_a7a8823c404228ab;

function __wbg_name_3a6997ca78d6bcd0(arg0) {
    return addHeapObject(getObject(arg0).name);
}

__exports.__wbg_name_3a6997ca78d6bcd0 = __wbg_name_3a6997ca78d6bcd0;

function __wbg_next_c004b8a85ecf4b77(arg0, exnptr) {
    try {
        return addHeapObject(getObject(arg0).next());
    } catch (e) {
        handleError(exnptr, e);
    }
}

__exports.__wbg_next_c004b8a85ecf4b77 = __wbg_next_c004b8a85ecf4b77;

function __wbg_done_178d004be150c0b1(arg0) {
    return getObject(arg0).done;
}

__exports.__wbg_done_178d004be150c0b1 = __wbg_done_178d004be150c0b1;

function __wbg_value_43d4ee3a28fa5f46(arg0) {
    return addHeapObject(getObject(arg0).value);
}

__exports.__wbg_value_43d4ee3a28fa5f46 = __wbg_value_43d4ee3a28fa5f46;

function __wbg_toString_4964e720c1c2dd0b(arg0) {
    return addHeapObject(getObject(arg0).toString());
}

__exports.__wbg_toString_4964e720c1c2dd0b = __wbg_toString_4964e720c1c2dd0b;

function __wbg_get_44104914d11d4644(arg0, arg1, exnptr) {
    try {
        return addHeapObject(Reflect.get(getObject(arg0), getObject(arg1)));
    } catch (e) {
        handleError(exnptr, e);
    }
}

__exports.__wbg_get_44104914d11d4644 = __wbg_get_44104914d11d4644;

function __wbg_iterator_4c465b932aa93752() {
    return addHeapObject(Symbol.iterator);
}

__exports.__wbg_iterator_4c465b932aa93752 = __wbg_iterator_4c465b932aa93752;

let cachedTextDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

function __wbindgen_string_new(p, l) { return addHeapObject(getStringFromWasm(p, l)); }

__exports.__wbindgen_string_new = __wbindgen_string_new;

function __wbindgen_is_object(i) {
    const val = getObject(i);
    return typeof(val) === 'object' && val !== null ? 1 : 0;
}

__exports.__wbindgen_is_object = __wbindgen_is_object;

function __wbindgen_is_function(i) { return typeof(getObject(i)) === 'function' ? 1 : 0; }

__exports.__wbindgen_is_function = __wbindgen_is_function;

function __wbindgen_string_get(i, len_ptr) {
    let obj = getObject(i);
    if (typeof(obj) !== 'string') return 0;
    const ptr = passStringToWasm(obj);
    getUint32Memory()[len_ptr / 4] = WASM_VECTOR_LEN;
    return ptr;
}

__exports.__wbindgen_string_get = __wbindgen_string_get;

function __wbindgen_json_parse(ptr, len) { return addHeapObject(JSON.parse(getStringFromWasm(ptr, len))); }

__exports.__wbindgen_json_parse = __wbindgen_json_parse;

function __wbindgen_json_serialize(idx, ptrptr) {
    const ptr = passStringToWasm(JSON.stringify(getObject(idx)));
    getUint32Memory()[ptrptr / 4] = ptr;
    return WASM_VECTOR_LEN;
}

__exports.__wbindgen_json_serialize = __wbindgen_json_serialize;

function __wbindgen_rethrow(idx) { throw takeObject(idx); }

__exports.__wbindgen_rethrow = __wbindgen_rethrow;

function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

__exports.__wbindgen_throw = __wbindgen_throw;

function freeLincolnIntepretor(ptr) {

    wasm.__wbg_lincolnintepretor_free(ptr);
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
        freeLincolnIntepretor(ptr);
    }

    /**
    * @returns {LincolnIntepretor}
    */
    static new() {
        return LincolnIntepretor.__wrap(wasm.lincolnintepretor_new());
    }
    /**
    * @param {any} prog
    * @returns {void}
    */
    set_program(prog) {
        try {
            return wasm.lincolnintepretor_set_program(this.ptr, addBorrowedObject(prog));

        } finally {
            heap[stack_pointer++] = undefined;

        }

    }
    /**
    * @returns {any}
    */
    get_program() {
        return takeObject(wasm.lincolnintepretor_get_program(this.ptr));
    }
    /**
    * @param {string} jmplabel
    * @param {string} jmpcont
    * @param {number} per
    * @returns {void}
    */
    jmp(jmplabel, jmpcont, per) {
        const ptr0 = passStringToWasm(jmplabel);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm(jmpcont);
        const len1 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_jmp(this.ptr, ptr0, len0, ptr1, len1, per);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);
            wasm.__wbindgen_free(ptr1, len1 * 1);

        }

    }
    /**
    * @param {string} calllabel
    * @param {string} callee
    * @param {number} callcnt
    * @param {string} callcont
    * @returns {void}
    */
    call(calllabel, callee, callcnt, callcont) {
        const ptr0 = passStringToWasm(calllabel);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm(callee);
        const len1 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm(callcont);
        const len3 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_call(this.ptr, ptr0, len0, ptr1, len1, callcnt, ptr3, len3);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);
            wasm.__wbindgen_free(ptr1, len1 * 1);
            wasm.__wbindgen_free(ptr3, len3 * 1);

        }

    }
    /**
    * @param {string} retlabel
    * @param {number} variant
    * @returns {void}
    */
    ret(retlabel, variant) {
        const ptr0 = passStringToWasm(retlabel);
        const len0 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_ret(this.ptr, ptr0, len0, variant);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);

        }

    }
    /**
    * @param {string} grouplabel
    * @param {any[]} elements
    * @returns {void}
    */
    group(grouplabel, elements) {
        const ptr0 = passStringToWasm(grouplabel);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm(elements);
        const len1 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_group(this.ptr, ptr0, len0, ptr1, len1);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);

        }

    }
    /**
    * @param {string} exportlabel
    * @returns {void}
    */
    set_export(exportlabel) {
        const ptr0 = passStringToWasm(exportlabel);
        const len0 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_set_export(this.ptr, ptr0, len0);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);

        }

    }
    /**
    * @param {string} deletelabel
    * @returns {void}
    */
    delete(deletelabel) {
        const ptr0 = passStringToWasm(deletelabel);
        const len0 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_delete(this.ptr, ptr0, len0);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);

        }

    }
    /**
    * @param {any} externs
    * @returns {void}
    */
    compile(externs) {
        try {
            return wasm.lincolnintepretor_compile(this.ptr, addBorrowedObject(externs));

        } finally {
            heap[stack_pointer++] = undefined;

        }

    }
    /**
    * @param {string} entry
    * @param {number} variant
    * @param {any} values
    * @param {boolean} step
    * @returns {void}
    */
    run(entry, variant, values, step) {
        const ptr0 = passStringToWasm(entry);
        const len0 = WASM_VECTOR_LEN;
        try {
            return wasm.lincolnintepretor_run(this.ptr, ptr0, len0, variant, addBorrowedObject(values), step);

        } finally {
            wasm.__wbindgen_free(ptr0, len0 * 1);
            heap[stack_pointer++] = undefined;

        }

    }
    /**
    * @returns {boolean}
    */
    step() {
        return (wasm.lincolnintepretor_step(this.ptr)) !== 0;
    }
    /**
    * @returns {any}
    */
    get_context() {
        return takeObject(wasm.lincolnintepretor_get_context(this.ptr));
    }
}

__exports.LincolnIntepretor = LincolnIntepretor;

function freeLincolnJsValue(ptr) {

    wasm.__wbg_lincolnjsvalue_free(ptr);
}
/**
*/
export class LincolnJsValue {

    free() {
        const ptr = this.ptr;
        this.ptr = 0;
        freeLincolnJsValue(ptr);
    }

}

__exports.LincolnJsValue = LincolnJsValue;

function __wbindgen_object_clone_ref(idx) {
    return addHeapObject(getObject(idx));
}

__exports.__wbindgen_object_clone_ref = __wbindgen_object_clone_ref;

function __wbindgen_object_drop_ref(i) { dropObject(i); }

__exports.__wbindgen_object_drop_ref = __wbindgen_object_drop_ref;

function init(module_or_path, maybe_memory) {
    let result;
    const imports = { './lincoln_wasm': __exports };
    if (module_or_path instanceof URL || typeof module_or_path === 'string' || module_or_path instanceof Request) {

        const response = fetch(module_or_path);
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

        result = WebAssembly.instantiate(module_or_path, imports)
        .then(instance => {
            return { instance, module: module_or_path };
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

