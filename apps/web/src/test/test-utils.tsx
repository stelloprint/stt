import {
	createRootRoute,
	createRoute,
	createRouter,
	RouterProvider,
} from "@tanstack/react-router";
import {
	type RenderOptions,
	render as rtlRender,
} from "@testing-library/react";
import type { ReactNode } from "react";

const rootRoute = createRootRoute({
	component: () => <div>Root</div>,
});

const indexRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/",
	component: () => <div>Index</div>,
});

const logsRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/logs",
	component: () => <div>Logs</div>,
});

const settingsRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/settings",
	component: () => <div>Settings</div>,
});

const recordRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/record",
	component: () => <div>Record</div>,
});

const routeTree = rootRoute.addChildren([
	indexRoute,
	logsRoute,
	settingsRoute,
	recordRoute,
]);

export const testRouter = createRouter({
	routeTree,
	defaultPreload: "intent",
});

export function render(
	ui: ReactNode,
	{
		route = "/",
		...options
	}: Omit<RenderOptions, "wrapper"> & { route?: string } = {}
) {
	testRouter.navigate({ to: route, from: "/" });
	const result = rtlRender(<RouterProvider router={testRouter} />, options);
	return result;
}

export * from "@testing-library/react";
