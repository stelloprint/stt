import { describe, expect, it } from "vitest";

const routes = {
	home: "/",
	logs: "/logs",
	settings: "/settings",
	record: "/record",
};

const buildPath = (path: string, params: Record<string, string>) => {
	let result = path;
	for (const [key, value] of Object.entries(params)) {
		result = result.split(`$${key}`).join(value);
	}
	return result;
};

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

const resolvePath = (from: string, to: string) => {
	if (to.startsWith("/")) {
		return to;
	}
	const fromParts = from.split("/");
	fromParts.pop();
	return [...fromParts, to].join("/");
};

const isValidPath = (path: string) => {
	return path.startsWith("/") && path.length > 1;
};

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

describe("Route Navigation", () => {
	describe("Route Definitions", () => {
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

		it("should have all required routes defined", () => {
			const requiredRoutes = ["/", "/logs", "/settings", "/record"];
			const definedRoutes = Object.values(routes);

			requiredRoutes.forEach((route) => {
				expect(definedRoutes).toContain(route);
			});
		});
	});

	describe("Path Building", () => {
		it("should build route path with single param", () => {
			expect(buildPath("/users/$userId", { userId: "123" })).toBe("/users/123");
		});

		it("should build route path with multiple params", () => {
			expect(
				buildPath("/users/$userId/posts/$postId", { userId: "1", postId: "2" })
			).toBe("/users/1/posts/2");
		});

		it("should handle missing params gracefully", () => {
			expect(buildPath("/users/$userId", {})).toBe("/users/$userId");
		});

		it("should replace all occurrences of param", () => {
			expect(buildPath("/$a/$a", { a: "x" })).toBe("/x/x");
		});
	});

	describe("Route Matching", () => {
		it("should match route with param", () => {
			expect(matchRoute("/users/123", "/users/$userId")).toBe(true);
		});

		it("should match exact route", () => {
			expect(matchRoute("/logs", "/logs")).toBe(true);
			expect(matchRoute("/settings", "/settings")).toBe(true);
			expect(matchRoute("/record", "/record")).toBe(true);
		});

		it("should not match route with wrong segment count", () => {
			expect(matchRoute("/users/123", "/users")).toBe(false);
			expect(matchRoute("/users", "/users/123")).toBe(false);
		});

		it("should not match route with different paths", () => {
			expect(matchRoute("/logs", "/settings")).toBe(false);
			expect(matchRoute("/settings", "/record")).toBe(false);
		});
	});

	describe("Route Resolution", () => {
		it("should resolve absolute paths directly", () => {
			expect(resolvePath("/logs", "/settings")).toBe("/settings");
			expect(resolvePath("/settings", "/record")).toBe("/record");
		});

		it("should resolve relative paths from parent", () => {
			expect(resolvePath("/logs", "search")).toBe("/search");
			expect(resolvePath("/settings", "profile")).toBe("/profile");
		});

		it("should resolve nested relative paths", () => {
			expect(resolvePath("/logs/filter", "new")).toBe("/logs/new");
			expect(resolvePath("/settings/security", "advanced")).toBe(
				"/settings/advanced"
			);
		});
	});

	describe("Path Validation", () => {
		it("should validate correct paths", () => {
			expect(isValidPath("/logs")).toBe(true);
			expect(isValidPath("/settings")).toBe(true);
			expect(isValidPath("/record")).toBe(true);
		});

		it("should reject root path as invalid", () => {
			expect(isValidPath("/")).toBe(false);
		});

		it("should reject empty paths", () => {
			expect(isValidPath("")).toBe(false);
		});
	});

	describe("Parameter Extraction", () => {
		it("should extract single param", () => {
			expect(extractParams("/users/123", "/users/$userId")).toEqual({
				userId: "123",
			});
		});

		it("should extract multiple params", () => {
			expect(
				extractParams("/users/123/posts/456", "/users/$userId/posts/$postId")
			).toEqual({
				userId: "123",
				postId: "456",
			});
		});

		it("should return empty object for static path", () => {
			expect(extractParams("/logs", "/logs")).toEqual({});
		});

		it("should not extract non-param segments", () => {
			expect(extractParams("/users/admin", "/users/$userId")).toEqual({
				userId: "admin",
			});
		});
	});

	describe("Navigation", () => {
		it("should use route as key for navigation", () => {
			const routeKey = "logs";
			const routePaths: Record<string, string> = routes;
			expect(routePaths[routeKey]).toBe("/logs");
		});

		it("should lookup settings route", () => {
			expect(routes.settings).toBe("/settings");
		});

		it("should lookup record route", () => {
			expect(routes.record).toBe("/record");
		});

		it("should handle unknown route keys", () => {
			const routePaths: Record<string, string> = routes;
			expect(routePaths["unknown"]).toBeUndefined();
		});
	});
});
