import { ReactQueryDevtools } from "@tanstack/react-query-devtools"
import type { TanStackDevtoolsReactPlugin } from "@tanstack/react-devtools"

const TanStackQueryDevtools: TanStackDevtoolsReactPlugin = {
  name: "TanStack Query",
  render: <ReactQueryDevtools />,
}

export default TanStackQueryDevtools
