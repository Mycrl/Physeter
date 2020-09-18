import { Not } from "./util"

/**
 * 任务回调处理
 * @desc 必须为async函数
 */
type Handle<T, U> = (task: T) => Promise<U>

/**
 * 队列项
 */
interface Value<T, U> {
    reject: (error: Error) => void
    resolve: (result: U) => void
    task: T
}

/**
 * 任务队列
 * @class
 */
export default class<T, U> {
    private queue: Array<Value<T, U>>
    private handle?: Handle<T, U>
    private worker: boolean
    private limit: number
    
    /**
     * @param limit 限制
     * @constructor
     */
    constructor (limit: number) {
        this.worker = false
        this.limit = limit
        this.queue = []
    }

    /**
     * 主循环
     * 执行任务队列
     */
    private async poll(): Promise<Not> {

        /**
         * 如果已经工作
         * 则不继续创建工作任务
         */
        if (this.worker) {
            return undefined
        }

        /**
         * 无限循环
         * 排空任务列表
         */
        this.worker = true
for(;;) {

        /**
         * 如果队列为空
         * 则跳出循环
         */
        if (this.queue.length === 0) {
            this.worker = false
            break
        }

        /**
         * 创建工作任务列表
         * 按限制数创建任务列表
         */
        const task_list = <Value<T, U>[]>
        new Array(this.limit)
        .fill(undefined)
        .map(() => this.queue.pop())
        .filter(task => task !== undefined)

        /**
         * 并行完成所有任务
         * 将任务回调传递给呼叫回调
         */
        await Promise.all(task_list.map(async ({ task, resolve, reject }) => {
            try { resolve(await this.handle!(task)) } catch (err) { reject(err) }
        }))
}
    }

    /**
     * 绑定处理
     * @param handle 处理
     */
    public bind(handle: Handle<T, U>): Not {
        this.handle = handle
    }
    
    /**
     * 呼叫调用
     * @param task 任务 
     */
    public call(task: T): Promise<U> {
        return new Promise((resolve, reject) => {
            this.queue.unshift({ task, resolve, reject })
            this.poll()
        })
    }
}
