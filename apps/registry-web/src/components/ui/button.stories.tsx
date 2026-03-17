import type { Meta, StoryObj } from "@storybook/react"
import { Download, Plus } from "lucide-react"
import { Button } from "./button"

const meta: Meta<typeof Button> = {
  title: "UI/Button",
  component: Button,
  parameters: { layout: "centered" },
  tags: ["autodocs"],
  argTypes: {
    variant: {
      control: "select",
      options: ["default", "outline", "secondary", "ghost", "destructive", "link"],
    },
    size: {
      control: "select",
      options: ["xs", "sm", "default", "lg", "icon", "icon-xs", "icon-sm", "icon-lg"],
    },
  },
}

export default meta
type Story = StoryObj<typeof Button>

export const Default: Story = {
  args: { children: "Button" },
}

export const Outline: Story = {
  args: { variant: "outline", children: "Outline" },
}

export const Secondary: Story = {
  args: { variant: "secondary", children: "Secondary" },
}

export const Ghost: Story = {
  args: { variant: "ghost", children: "Ghost" },
}

export const Destructive: Story = {
  args: { variant: "destructive", children: "Delete" },
}

export const WithIcon: Story = {
  args: { children: <><Plus />Add package</> },
}

export const IconOnly: Story = {
  args: { size: "icon", "aria-label": "Download", children: <Download /> },
}

export const Disabled: Story = {
  args: { children: "Disabled", disabled: true },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Button>Default</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Destructive</Button>
      <Button variant="link">Link</Button>
    </div>
  ),
}
