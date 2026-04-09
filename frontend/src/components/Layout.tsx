import { ReactNode } from 'react'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  return (
    <div className="flex h-screen bg-dark-950">
      <main className="flex-1 overflow-hidden">
        {children}
      </main>
    </div>
  )
}
