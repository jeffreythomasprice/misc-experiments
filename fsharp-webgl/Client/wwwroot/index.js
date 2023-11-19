/**
 * @param {HTMLCanvasElement} canvas 
 */
globalThis.init = async (canvas) => {
    try {
        const context = canvas.getContext(
            "webgl2",
            {
                powerPreference: "high-performance",
            }
        );

        await DotNet.invokeMethodAsync("Client", "init", DotNet.createJSObjectReference(context));

        const resize = async () => {
            try {
                canvas.width = window.innerWidth;
                canvas.height = window.innerHeight;
                await DotNet.invokeMethodAsync("Client", "resize", window.innerWidth, window.innerHeight);
            } catch (err) {
                console.error("error in resize", err);
            }
        };
        window.onresize = resize;
        await resize();

        /**
         * @param {number} time 
         */
        const animate = async (time) => {
            try {
                await DotNet.invokeMethodAsync("Client", "animate", time);
                requestAnimationFrame(animate);
            } catch (err) {
                console.error("error in animate", err);
            }
        };
        requestAnimationFrame(animate);
    } catch (err) {
        console.error("init failure", err);
    }
};

globalThis.getValue = (target, name) => target[name];

globalThis.arrayToFloat32Array = (array) => {
    const offset = array + 12;
    const length = Module.HEAP32[offset >> 2];
    return new Float32Array(Module.HEAPF32.buffer, offset + 4, length);
};