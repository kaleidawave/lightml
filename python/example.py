from wasmtime import Memory, Store, Module, Instance, Func, FuncType, Table

store = Store()
module = Module.from_file(store.engine, "./pkg/build_bg.wasm")

# Not sure
def __wbindgen_init_externref_table():
    table: Table = instance.exports(store)["__wbindgen_export_0"]
    offset = table.grow(store, 4, 0)
    # table.set(store, 0, None)
    # table.set(store, offset+0, None)
    # table.set(store, offset+1, None)
    # table.set(store, offset+2, True)
    # table.set(store, offset+3, False)

instance = Instance(store, module, [Func(store, FuncType([], []), __wbindgen_init_externref_table)])
exports = instance.exports(store)
memory: Memory = exports["memory"]

exports["__wbindgen_start"](store)

# print([*.keys()])

WASM_VECTOR_LEN = 0
cachedUint8ArrayMemory0 = None

def retrieve(content, query):
    deferred3_0 = None
    deferred3_1 = None
    try:
        malloc = exports["__wbindgen_malloc"] # , exports["__wbindgen_realloc"]
        len0 = len(content)
        ptr0 = malloc(store, len0, 1) >> 0
        content = bytes(content, "utf-8")
        if content.find(0x7F) != -1:
            raise Exception("content")
        memory.write(store, content, ptr0)

        len1 = len(query)
        ptr1 = malloc(store, len1, 1) >> 0
        query = bytes(query, "utf-8")
        if query.find(0x7F) != -1:
            raise Exception("query")
        memory.write(store, query, ptr1)

        ret = exports["retrieve"](store, ptr0, len0, ptr1, len1)
        deferred3_0 = ret[0]
        deferred3_1 = ret[1]
        out = memory.read(store, ret[0], ret[0] + ret[1])
        return str(out, "utf-8")
    finally:
        if deferred3_0 is not None:
            exports["__wbindgen_free"](store, deferred3_0, deferred3_1, 1)

input1 = "<html><body><h1>Hi</h1></body></html>"
selector1 = "all h1\0text"
print("out1", retrieve(input1, selector1))

print("\nnext\n")

m = open("./private/corpus/bbcnews.html")
input1 = m.read()
selector1 = "single #nations-news-uk\0all a[href^='/news/articles']\0text"
print("out2", retrieve(input1, selector1))
