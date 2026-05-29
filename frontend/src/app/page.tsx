import Link from "next/link";

export default function Home() {
  return (
    <div className="bg-slate-950 text-slate-100 min-h-[calc(100vh-3.5rem)] bg-cyber-grid animate-scanline flex flex-col items-center justify-center p-6 relative">
      {/* Web3 corner decorations */}
      <div className="absolute top-8 left-8 text-[10px] font-mono text-cyan-500/40 hidden md:block space-y-1">
        <div>[SYS_STATUS: ACTIVE]</div>
        <div>[ORACLE_LINK: SUCCESS]</div>
        <div>[NODE: MUTAO-MAINNET]</div>
        <div className="text-cyan-500/25">0x7a3f...e2b1</div>
      </div>
      <div className="absolute bottom-8 right-8 text-[10px] font-mono text-cyan-500/40 hidden md:block text-right space-y-1">
        <div>[BLOCK: #18,429,073]</div>
        <div>[SYNC: 100%]</div>
        <div>[CHAIN: ETH + SOL]</div>
        <div className="text-amber-500/25">0xff99...0x00</div>
      </div>

      {/* Crosshair decorations */}
      <div className="absolute top-1/4 left-12 text-cyan-500/10 text-2xl hidden lg:block">+</div>
      <div className="absolute top-1/3 right-16 text-cyan-500/10 text-2xl hidden lg:block">+</div>
      <div className="absolute bottom-1/4 left-20 text-amber-500/10 text-2xl hidden lg:block">+</div>

      {/* Main content */}
      <div className="max-w-2xl text-center relative z-10">
        {/* Status tag */}
        <div className="inline-flex items-center gap-2 mb-8 px-4 py-1.5 clip-mech-btn bg-cyan-500/5 border border-cyan-500/20">
          <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-pulse" />
          <span className="text-[10px] font-mono text-cyan-400/70 tracking-[0.2em]">SYSTEM READY</span>
        </div>

        {/* Title */}
        <h1 className="text-7xl md:text-8xl font-black tracking-widest mb-3 glitch">
          <span className="bg-gradient-to-r from-cyan-400 via-amber-400 to-orange-500 bg-clip-text text-transparent glow-text-cyan">
            木桃
          </span>
        </h1>
        <p className="text-xs font-mono text-cyan-500/50 mb-4 tracking-[0.3em]">MUTAO PROTOCOL v0.1</p>

        {/* Slogan */}
        <div className="font-mono text-slate-400 text-lg mb-2">
          <span className="text-cyan-500/40 text-xs mr-2">// INITIALIZING:</span>
          投我以木桃，报之以琼瑶
        </div>
        <p className="text-sm text-slate-500 font-mono mb-10">
          AI 撮合 + Web3 溯源的免现金实体易物平台
        </p>

        {/* Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link
            href="/items"
            className="btn-flow clip-mech-panel inline-flex items-center justify-center px-10 py-3.5 text-sm font-bold bg-gradient-to-r from-amber-500 to-orange-600 text-slate-950 glow-orange hover:brightness-110 transition-all tracking-wider"
          >
            浏览物品
          </Link>
          <Link
            href="/auth/register"
            className="clip-mech-panel inline-flex items-center justify-center px-10 py-3.5 text-sm font-bold border border-cyan-400 text-cyan-400 bg-cyan-950/20 hover:bg-cyan-400 hover:text-slate-950 transition-all tracking-wider"
          >
            开始交换
          </Link>
        </div>

        {/* Bottom tech line */}
        <div className="mt-14 flex items-center gap-4 justify-center text-slate-600 text-[10px] font-mono">
          <div className="w-20 h-px bg-gradient-to-r from-transparent to-cyan-500/30" />
          <span className="text-cyan-500/30">DEVELOPED ON RUST + AXUM | SOLIDITY | NEXT.JS</span>
          <div className="w-20 h-px bg-gradient-to-l from-transparent to-cyan-500/30" />
        </div>
      </div>
    </div>
  );
}
