import Link from "next/link";

export default function Home() {
  return (
    <div className="flex min-h-[calc(100vh-3.5rem)] flex-col items-center justify-center p-6 md:p-24">
      <div className="max-w-2xl text-center">
        <h1 className="text-4xl md:text-6xl font-bold mb-4 bg-gradient-to-r from-amber-400 to-orange-500 bg-clip-text text-transparent">
          木桃
        </h1>
        <p className="text-lg md:text-xl text-gray-400 mb-2">
          投我以木桃，报之以琼瑶
        </p>
        <p className="text-base text-gray-500 mb-10">
          AI 撮合 + Web3 溯源的免现金实体易物平台
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link
            href="/items"
            className="inline-flex items-center justify-center px-8 py-3 text-sm font-medium bg-amber-500 hover:bg-amber-600 text-black rounded-lg transition-colors shadow-lg shadow-amber-500/20"
          >
            浏览物品
          </Link>
          <Link
            href="/auth/register"
            className="inline-flex items-center justify-center px-8 py-3 text-sm font-medium border border-white/20 hover:border-white/40 text-gray-300 hover:text-white rounded-lg transition-colors"
          >
            开始交换
          </Link>
        </div>
      </div>
    </div>
  );
}
