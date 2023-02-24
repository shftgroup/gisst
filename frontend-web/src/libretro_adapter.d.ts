declare function startRetroArch(canvas:HTMLCanvasElement, arguments:string[], initialized_cb:() => void);
declare function loadRetroArch(core:string, loaded_cb:() => void);
declare function retroArchSend(msg:string);
