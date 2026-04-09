/**
 * Accessibility hooks for keyboard navigation
 */

import * as React from 'react'

// ============================================================================
// Keyboard Navigation for Dropdown Menus
// ============================================================================

export interface UseKeyboardDropdownOptions {
  isOpen: boolean
  onClose: () => void
  itemCount: number
  onSelect?: (index: number) => void
}

export interface UseKeyboardDropdownReturn {
  triggerProps: {
    'aria-expanded': boolean
    'aria-haspopup': string
    onKeyDown: (e: React.KeyboardEvent) => void
  }
  menuProps: {
    role: string
    onKeyDown: (e: React.KeyboardEvent) => void
  }
  getItemProps: (index: number) => {
    role: string
    tabIndex: number
    'aria-selected': boolean
    onKeyDown: (e: React.KeyboardEvent) => void
    onClick: () => void
    ref: (el: HTMLElement | null) => void
  }
  activeIndex: number
  setActiveIndex: (index: number) => void
}

/**
 * Hook for accessible dropdown menus with full keyboard navigation
 *
 * Supports: ArrowDown, ArrowUp, Home, End, Escape, Enter, Space, Tab
 */
export function useKeyboardDropdown({
  isOpen,
  onClose,
  itemCount,
  onSelect,
}: UseKeyboardDropdownOptions): UseKeyboardDropdownReturn {
  const [activeIndex, setActiveIndex] = React.useState(-1)
  const itemRefs = React.useRef<(HTMLElement | null)[]>([])

  // Reset index when menu closes
  React.useEffect(() => {
    if (!isOpen) {
      setActiveIndex(-1)
    } else {
      setActiveIndex(0)
    }
  }, [isOpen])

  // Focus active item when index changes
  React.useEffect(() => {
    if (isOpen && activeIndex >= 0 && activeIndex < itemCount) {
      const el = itemRefs.current[activeIndex]
      if (el) {
        el.focus()
      }
    }
  }, [activeIndex, isOpen, itemCount])

  const handleTriggerKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        if (!isOpen) {
          onClose() // Will be toggled by parent
        }
        break
      case 'Escape':
        e.preventDefault()
        onClose()
        break
      case 'Enter':
      case ' ':
        e.preventDefault()
        if (isOpen && activeIndex >= 0) {
          onSelect?.(activeIndex)
        }
        break
    }
  }

  const handleMenuKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setActiveIndex((prev) => (prev + 1) % itemCount)
        break
      case 'ArrowUp':
        e.preventDefault()
        setActiveIndex((prev) => (prev - 1 + itemCount) % itemCount)
        break
      case 'Home':
        e.preventDefault()
        setActiveIndex(0)
        break
      case 'End':
        e.preventDefault()
        setActiveIndex(itemCount - 1)
        break
      case 'Enter':
      case ' ':
        e.preventDefault()
        if (activeIndex >= 0) {
          onSelect?.(activeIndex)
        }
        break
      case 'Escape':
        e.preventDefault()
        onClose()
        break
      case 'Tab':
        // Close on tab out
        onClose()
        break
    }
  }

  const getItemProps = (index: number) => ({
    role: 'option' as const,
    tabIndex: index === activeIndex ? 0 : -1,
    'aria-selected': index === activeIndex,
    onKeyDown: handleMenuKeyDown,
    onClick: () => onSelect?.(index),
    ref: (el: HTMLElement | null) => {
      itemRefs.current[index] = el
    },
  })

  return {
    triggerProps: {
      'aria-expanded': isOpen,
      'aria-haspopup': 'listbox' as const,
      onKeyDown: handleTriggerKeyDown,
    },
    menuProps: {
      role: 'listbox',
      onKeyDown: handleMenuKeyDown,
    },
    getItemProps,
    activeIndex,
    setActiveIndex,
  }
}

// ============================================================================
// Focus Trap
// ============================================================================

export interface UseFocusTrapOptions {
  isActive: boolean
  initialFocus?: React.RefObject<HTMLElement>
  onEscape?: () => void
}

/**
 * Hook to trap focus within a container (for modals, dialogs, popups)
 */
export function useFocusTrap({
  isActive,
  initialFocus,
  onEscape,
}: UseFocusTrapOptions) {
  const containerRef = React.useRef<HTMLElement>(null)

  React.useEffect(() => {
    if (!isActive) return

    const container = containerRef.current
    if (!container) return

    // Focus initial element or first focusable element
    const focusable = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )
    const firstFocusable = focusable[0]
    const lastFocusable = focusable[focusable.length - 1]

    if (initialFocus?.current) {
      initialFocus.current.focus()
    } else if (firstFocusable) {
      firstFocusable.focus()
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onEscape?.()
        return
      }

      if (e.key !== 'Tab') return

      // Trap focus
      if (e.shiftKey) {
        if (document.activeElement === firstFocusable) {
          e.preventDefault()
          lastFocusable?.focus()
        }
      } else {
        if (document.activeElement === lastFocusable) {
          e.preventDefault()
          firstFocusable?.focus()
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [isActive, initialFocus, onEscape])

  return containerRef
}

// ============================================================================
// Live Region (Screen Reader Announcements)
// ============================================================================

/**
 * Announce a message to screen readers via aria-live region
 */
export function useAnnouncer() {
  const [message, setMessage] = React.useState('')

  React.useEffect(() => {
    if (!message) return
    // Clear after 3 seconds
    const timer = setTimeout(() => setMessage(''), 3000)
    return () => clearTimeout(timer)
  }, [message])

  // Create live region element
  const LiveRegion = () => (
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className="sr-only"
    >
      {message}
    </div>
  )

  return { announce: setMessage, LiveRegion }
}
