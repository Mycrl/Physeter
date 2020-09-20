import Kernel from "./kernel/kernel"
import fs from "fs"

//fs.unlinkSync("./.static/index")
//fs.unlinkSync("./.static/0.track")

const kernel = new Kernel({
    directory: "./.static"
})

kernel.init().then(async () => {
//    const write = kernel.write("test")
//    if (write) {
//        fs.createReadStream("./test.mp4").pipe(write).on("finish", () => {
//            console.log("Ok")
//        })
//    }

    console.time("read")
     const read = await kernel.read("test")
     if (read) {
         read.pipe(fs.createWriteStream("./out.mp4")).on("finish", () => {
             console.timeEnd("read")
         })
     }
})
