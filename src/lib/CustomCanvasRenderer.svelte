<script>
    import { invoke } from '@tauri-apps/api/tauri';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte'
    import { appWindow } from '@tauri-apps/api/window';
    
    let canvas
    let context
    let resizeTimeout

    const fontHeight = 10
    let fontWidth

    const writeToPty = async (e) => {
        let code = e.key.charCodeAt(0)
        if (code === 69 || code === 66) {
            code = e.keyCode
        }
        await invoke("write_to_pty", {data: code})
    }

    const resize = async () => {
        clearTimeout(resizeTimeout)
        setTimeout(async () => {
            const winSize = await appWindow.innerSize();
            canvas.width = winSize.width;
            canvas.height = winSize.height;

            const cols = window.Math.round((canvas.width / 2) / fontWidth)
            const rows = window.Math.round((canvas.height / 2) / fontHeight)

            await invoke("resize_term", {
                rows, cols
            })
        }, 500)
    }

    onMount(async () => {
        context = canvas.getContext('2d')
        const metrics = context.measureText("W")
        fontWidth = metrics.width

        resize()

        listen("term_data", (e) => {
            const offscreenCanvas = document.createElement("canvas")
            offscreenCanvas.width = canvas.width;
            offscreenCanvas.height = canvas.height;
            const offsetContext = offscreenCanvas.getContext('2d')

            for (const cell of e.payload) {
                let x = (cell.column + 1.0) * fontWidth;
                let cell_line = cell.line + cell.display_offset;
                let y = (cell_line + 1.0) * fontHeight;
                offsetContext.fillStyle = `rgba(${cell.bg.r},${cell.bg.g},${cell.bg.b},${cell.bg.a})`;
                offsetContext.fillRect(x, y, fontWidth, fontHeight);
                if (cell.content !== '\t' || cell.content !== ' ') {
                    offsetContext.fillStyle = `rgba(${cell.fg.r},${cell.fg.g},${cell.fg.b},${cell.fg.a})`;
                    offsetContext.fillText(cell.content, x, y)
                }
            }

            context.drawImage(offscreenCanvas, 0, 0)
        })

        await invoke("init_read")
    })
</script>

<canvas bind:this={canvas} />
<svelte:window on:keydown|preventDefault={writeToPty} on:resize={() => resize()} />
