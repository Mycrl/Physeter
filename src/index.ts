import Index from "./kernel/index/index"

const index = new Index({
    directory: "./.static",
    track_size: 0,
    chunk_size: 0,
    max_memory: 0
})

void (async () => {
    await index.initialize()
    for (let i = 0; i < 100000; i ++) {
       await index.set({
           name: String(i),
           start_chunk: [1, i],
           start_matedata: [2, i]
       })
    }

    console.time("find")
    console.log(await index.get("9999"))
    console.timeEnd("find")
})()

setInterval(() => {
    
}, 100000)