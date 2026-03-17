import type { Meta, StoryObj } from "@storybook/react"
import { fn } from "@storybook/test"
import { DependencyGraph } from "./DependencyGraph"

// Mock TanStack Router's useNavigate so the component renders outside the router
import * as TanStackRouter from "@tanstack/react-router"
const useNavigateMock = fn().mockReturnValue(fn())
TanStackRouter.useNavigate = () => useNavigateMock() as ReturnType<typeof TanStackRouter.useNavigate>

const meta: Meta<typeof DependencyGraph> = {
  title: "Components/DependencyGraph",
  component: DependencyGraph,
  parameters: { layout: "centered" },
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div style={{ width: 500 }}>
        <Story />
      </div>
    ),
  ],
}

export default meta
type Story = StoryObj<typeof DependencyGraph>

export const Default: Story = {
  args: {
    packageName: "@tsx-pkg/with-auth",
    integratesWith: [
      "@tsx-pkg/tanstack-start",
      "@tsx-pkg/drizzle-postgres",
      "@tsx-pkg/basic-crud",
    ],
  },
}

export const ManyDependencies: Story = {
  args: {
    packageName: "@tsx-pkg/full-saas",
    integratesWith: [
      "@tsx-pkg/with-auth",
      "@tsx-pkg/drizzle-postgres",
      "@tsx-pkg/with-shadcn",
      "@tsx-pkg/basic-crud",
      "@tsx-pkg/tanstack-start",
    ],
  },
}

export const NoDependencies: Story = {
  args: {
    packageName: "@tsx-pkg/with-shadcn",
    integratesWith: [],
  },
}

export const SingleDependency: Story = {
  args: {
    packageName: "@tsx-pkg/basic-crud",
    integratesWith: ["@tsx-pkg/drizzle-postgres"],
  },
}
