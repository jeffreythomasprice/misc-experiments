import asyncio
import atexit
import logging
import time

import glfw
import wgpu
from rendercanvas.glfw import enable_glfw, get_glfw_present_methods
from wgpu.backends.wgpu_native import GPUCanvasContext

logging.basicConfig(
    format="%(asctime)s %(levelname)-5s %(name)s: %(message)s",
    datefmt="%Y-%m-%dT%H:%M:%S",
    handlers=[logging.StreamHandler()],
    level=logging.INFO,
)

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)


class GlfwCanvas:
    def __init__(self):
        self.window = glfw.create_window(800, 600, "Experiment", None, None)
        self.context = GPUCanvasContext(self, get_glfw_present_methods(self.window))
        glfw.set_key_callback(self.window, self.key_event)

    def get_physical_size(self):
        x, y = glfw.get_framebuffer_size(self.window)
        return int(x), int(y)

    def get_context(self, _kind):
        return self.context

    def destroy(self):
        glfw.destroy_window(self.window)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()

    def key_event(self, _window, key, _unknown, pressed, _unknown2):
        pressed = bool(pressed)
        match key, pressed:
            case (glfw.KEY_ESCAPE, False):
                glfw.set_window_should_close(self.window, True)


async def init_wgpu(canvas: GlfwCanvas):
    adapter = await wgpu.gpu.request_adapter_async(
        power_preference=wgpu.enums.PowerPreference.high_performance,  # pyright: ignore[reportArgumentType]
        canvas=canvas,
    )
    logger.info(f"adapter: {adapter.info}")

    device = await adapter.request_device_async()
    logger.info("created device")


async def main():
    enable_glfw()
    atexit.register(glfw.terminate)

    glfw.window_hint(glfw.CLIENT_API, glfw.NO_API)
    glfw.window_hint(glfw.RESIZABLE, True)

    with GlfwCanvas() as canvas:
        await init_wgpu(canvas)

        last_frame_time = time.perf_counter()
        frame_count = 0

        while not glfw.window_should_close(canvas.window):
            glfw.poll_events()

            # TODO render here
            canvas.context.present()

            frame_count += 1
            etime = time.perf_counter() - last_frame_time
            if etime > 1:
                logger.debug(f"FPS: {frame_count / etime:0.1f}")
                last_frame_time, frame_count = time.perf_counter(), 0


if __name__ == "__main__":
    asyncio.run(main())
