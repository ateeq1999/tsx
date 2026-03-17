import type { Preview } from "@storybook/react"
import { ThemeProvider } from "next-themes"
import "../src/app.css"

const preview: Preview = {
  parameters: {
    backgrounds: {
      default: "light",
      values: [
        { name: "light", value: "#ffffff" },
        { name: "dark", value: "#0d1117" },
      ],
    },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story) => (
      <ThemeProvider attribute="class" defaultTheme="light">
        <div className="p-4">
          <Story />
        </div>
      </ThemeProvider>
    ),
  ],
}

export default preview
