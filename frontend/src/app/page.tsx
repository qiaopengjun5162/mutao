import { Button } from "@/components/ui/button"

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <h1 className="text-4xl font-bold mb-4">木桃 Mutao</h1>
      <p className="text-xl text-muted-foreground mb-8">
        AI 撮合 + Web3 溯源的免现金实体易物平台
      </p>
      <div className="flex gap-4">
        <Button>浏览物品</Button>
        <Button variant="outline">发布物品</Button>
      </div>
    </main>
  )
}
