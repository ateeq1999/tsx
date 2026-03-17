import type { Meta, StoryObj } from "@storybook/react"
import { Badge } from "./badge"

const meta: Meta<typeof Badge> = {
  title: "UI/Badge",
  component: Badge,
  parameters: { layout: "centered" },
  tags: ["autodocs"],
  argTypes: {
    variant: {
      control: "select",
      options: ["default", "secondary", "destructive", "outline", "ghost"],
    },
  },
}

export default meta
type Story = StoryObj<typeof Badge>

export const Default: Story = {
  args: { children: "typescript" },
}

export const Secondary: Story = {
  args: { variant: "secondary", children: "drizzle" },
}

export const Destructive: Story = {
  args: { variant: "destructive", children: "yanked" },
}

export const Outline: Story = {
  args: { variant: "outline", children: "v1.3.0" },
}

export const CapabilityTokens: Story = {
  render: () => (
    <div className="flex flex-wrap gap-1">
      {["auth", "session", "crud", "ui-components", "database", "billing"].map((cap) => (
        <Badge key={cap}>{cap}</Badge>
      ))}
    </div>
  ),
}
