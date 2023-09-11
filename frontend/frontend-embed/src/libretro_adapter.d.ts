declare interface LibretroModule extends EmscriptenModule {
  startRetroArch(canvas:HTMLCanvasElement, arguments:string[], initialized_cb:() => void);
  retroArchSend(msg:string);
  retroArchRecv():string;
}
declare function loadRetroArch(core:string, loaded_cb:(LibretroModule) => void);
