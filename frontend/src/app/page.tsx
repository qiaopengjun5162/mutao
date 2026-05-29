import Link from "next/link";

export default function Home() {
  return (
    <div className="grid-bg scanline min-h-[calc(100vh-3.5rem)] flex flex-col items-center justify-center p-6 relative">
      {/* Decorative corners */}
      <div className="absolute top-8 left-8 text-cyan/20 text-xs font-mono hidden md:block">
        <div>SYS.STATUS: ONLINE</div>
        <div>NODE: MUTAO-MAINNET</div>
        <div className="mt-2 text-cyan/30">{"0x7a3f...e2b1"}</div>
      </div>
      <div className="absolute bottom-8 right-8 text-cyan/20 text-xs font-mono hidden md:block text-right">
        <div>BLOCK: #18,429,073</div>
        <div>SYNC: 100%</div>
        <div className="mt-2 text-orange/30">{"// web3.vercel"}</div>
      </div>

      {/* Main content */}
      <div className="max-w-2xl text-center relative">
        {/* Status tag */}
        <div className="inline-flex items-center gap-2 mb-6 px-3 py-1 clip-cyber-sm bg-cyan/5 border border-cyan/20">
          <span className="w-1.5 h-1.5 rounded-full bg-cyan animate-pulse" />
          <span className="text-xs font-mono text-cyan/70 tracking-wider">SYSTEM READY</span>
        </div>

        {/* Title */}
        <h1 className="text-5xl md:text-7xl font-bold mb-4 glitch">
          <span className="bg-gradient-to-r from-cyan via-cyan to-orange bg-clip-text text-transparent">
            木桃
          </span>
        </h1>
        <p className="text-sm font-mono text-cyan/40 mb-2 tracking-widest">MUTAO PROTOCOL</p>
        <p className="text-lg md:text-xl text-slate-400 mb-2">
          投我以木桃，报之以琼瑶
        </p>
        <p className="text-sm text-slate-500 mb-10 font-mono">
          AI 撮合 + Web3 溯源的免现金实体易物平台
        </p>

        {/* Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link
            href="/items"
            className="btn-flow clip-cyber inline-flex items-center justify-center px-10 py-3.5 text-sm font-medium bg-cyan text-slate-950 transition-all duration-300 hover:bg-cyan/90 tracking-wider"
          >
            浏览物品
          </Link>
          <Link
            href="/auth/register"
            className="clip-cyber inline-flex items-center justify-center px-10 py-3.5 text-sm font-medium border border-orange/40 text-orange hover:bg-orange/10 glow-orange transition-all duration-300 tracking-wider"
          >
            开始交换
          </Link>
        </div>

        {/* Decorative line */}
        <div className="mt-12 flex items-center gap-4 justify-center text-slate-600 text-xs font-mono">
          <div className="w-16 h-px bg-gradient-to-r from-transparent to-cyan/30" />
          <span>DEVELOPED ON RUST + AXUM</span>
          <div className="w-16 h-px bg-gradient-to-l from-transparent to-cyan/30" />
        </div>
      </div>
    </div>
  );
}
