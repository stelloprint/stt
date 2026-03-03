import { describe, expect, it } from "vitest";

describe("Route Navigation", () => {
	const routes = {
		home: "/",
		logs: "/logs",
		settings: "/settings",
		record: "/record",
	};

	it("should have home route at root path", () => {
		expect(routes.home).toBe("/");
	});

	it("should have logs route at /logs path", () => {
		expect(routes.logs).toBe("/logs");
	});

	it("should have settings route at /settings path", () => {
		expect(routes.settings).toBe("/settings");
	});

	it("should have record route at /record path", () => {
		expect(routes.record).toBe("/record");
	});

	it("should build route path with params", () => {
		const buildPath = (path: string, params: Record<string, string>) => {
			let result = path;
			for (const [key, value] of Object.entries(params)) {
				result = result.replace(`$${key}`, value);
			}
			return result;
		};

		expect(buildPath("/users/$userId", { userId: "123" })).toBe("/users/123");
	});

	it("should match route with params", () => {
		const matchRoute = (path: string, pattern: string) => {
			const patternParts = pattern.split("/");
			const pathParts = path.split("/");

			if (patternParts.length !== pathParts.length) {
				return false;
			}

			return patternParts.every((part, i) => {
				if (part.startsWith("$")) {
					return true;
				}
				return part === pathParts[i];
			});
		};

		expect(matchRoute("/users/123", "/users/$userId")).toBe(true);
		expect(matchRoute("/logs", "/logs")).toBe(true);
		expect(matchRoute("/settings", "/settings")).toBe(true);
		expect(matchRoute("/record", "/record")).toBe(true);
	});

	it("should use route as key for navigation", () => {
		const routeKey = "logs";
		const routePaths: Record<string, string> = routes;
		expect(routePaths[routeKey]).toBe("/logs");
	});

	it("should resolve relative navigation paths", () => {
		const resolvePath = (from: string, to: string) => {
			if (to.startsWith("/")) {
				return to;
			}
			const fromParts = from.split("/");
			fromParts.pop();
			return [...fromParts, to].join("/");
		};

		expect(resolvePath("/logs", "/settings")).toBe("/settings");
		expect(resolvePath("/logs", "search")).toBe("/search");
	});

	it("should handle route path validation", () => {
		const isValidPath = (path: string) => {
			return path.startsWith("/") && path.length > 1;
		};

		expect(isValidPath("/logs")).toBe(true);
		expect(isValidPath("/settings")).toBe(true);
		expect(isValidPath("/record")).toBe(true);
		expect(isValidPath("/")).toBe(false);
	});

	it("should extract route params correctly", () => {
		const extractParams = (path: string, pattern: string) => {
			const patternParts = pattern.split("/");
			const pathParts = path.split("/");
			const params: Record<string, string> = {};

			patternParts.forEach((part, i) => {
				if (part.startsWith("$")) {
					params[part.slice(1)] = pathParts[i];
				}
			});

			return params;
		};

		expect(extractParams("/users/123", "/users/$userId")).toEqual({
			userId: "123",
		});
	});

	it("should have all required routes defined", () => {
		const requiredRoutes = ["/", "/logs", "/settings", "/record"];
		const definedRoutes = Object.values(routes);

		requiredRoutes.forEach((route) => {
			expect(definedRoutes).toContain(route);
		});
	});
});
