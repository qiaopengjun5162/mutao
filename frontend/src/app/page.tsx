import Link from "next/link";

export default function Home() {
  return (
    <div className="flex min-h-[calc(100vh-3.5rem)] flex-col items-center justify-center p-24">
      <h1 className="text-5xl font-bold mb-4">木桃 Mutao</h1>
      <p className="text-xl text-muted-foreground mb-2">
        投我以木桃，报之以琼瑶
      </p>
      <p className="text-lg text-muted-foreground mb-8">
        AI 撮合 + Web3 溯源的免现金实体易物平台
      </p>
      <div className="flex gap-4">
        <Link
          href="/items"
          className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium bg-primary text-primary-foreground shadow hover:bg-primary/90 h-10 px-8"
        >
          浏览物品
        </Link>
        <Link
          href="/items/new"
          className="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-10 px-8"
        >
          发布物品
        </Link>
      </div>
    </div>
  );
}
