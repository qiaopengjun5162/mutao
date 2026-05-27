import Link from "next/link";

export function Nav() {
  return (
    <nav className="border-b bg-background">
      <div className="container mx-auto flex h-14 items-center justify-between px-4">
        <Link href="/" className="text-lg font-bold">
          木桃 Mutao
        </Link>
        <div className="flex items-center gap-2">
          {[
            { href: "/items", label: "物品" },
            { href: "/items/new", label: "发布" },
            { href: "/demands", label: "意向" },
            { href: "/cycles", label: "交换" },
          ].map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="px-3 py-2 text-sm rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </div>
      </div>
    </nav>
  );
}
