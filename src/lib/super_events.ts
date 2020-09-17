type Handle<T, U> = (event: T) => Promise<U>
type Listener<T, U> = { [key: string]: Handle<T, U> }
type Listeners<T, U> = { [key: string]: Listener<T, U> }

/**
 * 超级事件类
 * @class
 */
export default class<T, U> {
    private listeners: Listeners<T, U>
    
    /**
     * @constructor
     */
    constructor() {
        this.listeners = {}
    }
    
    /**
     * 监听事件
     * @param space 命名空间
     * @param name 事件名
     * @param handle 事件处理
     */
    public on(space: string, name: string, handle: Handle<T, U>) {
        if (!this.listeners[space]) this.listeners[space] = {}
        this.listeners[space][name] = handle
    }
    
    /**
     * 呼叫事件
     * param space 命名空间
     * @param name 事件名
     * @param event 事件
     */
    public async call(space: string, name: string, event: T): Promise<U> {
        if (!this.listeners[space]) throw new Error("no listener")
        if (!this.listeners[space][name]) throw new Error("no event")
        return await this.listeners[space][name](event)
    }
}