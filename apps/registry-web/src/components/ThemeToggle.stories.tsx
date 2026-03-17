import type { Meta, StoryObj } from "@storybook/react"
import { ThemeToggle } from "./ThemeToggle"

const meta: Meta<typeof ThemeToggle> = {
  title: "Components/ThemeToggle",
  component: ThemeToggle,
  parameters: { layout: "centered" },
  tags: ["autodocs"],
}

export default meta
type Story = StoryObj<typeof ThemeToggle>

export const Default: Story = {}

export const InToolbar: Story = {
  render: () => (
    <div
      className="flex h-14 items-center gap-4 px-4"
      style={{ background: "var(--header-bg)", border: "1px solid var(--line)" }}
    >
      <span className="text-sm font-semibold">tsx registry</span>
      <div className="ml-auto">
        <ThemeToggle />
      </div>
    </div>
  ),
}
