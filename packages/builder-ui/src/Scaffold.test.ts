import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import Scaffold from "./Scaffold.svelte";

afterEach(cleanup);

describe("Builder bootstrap", () => {
  it("mounts the Svelte scaffold with a named heading and main landmark", () => {
    render(Scaffold);
    expect(screen.getByRole("main")).toBeDefined();
    expect(
      screen.getByRole("heading", { level: 1, name: "AFTERGLOW Builder" }),
    ).toBeDefined();
  });

  it("keeps builds unavailable until the build pipeline is implemented", () => {
    render(Scaffold);
    expect(
      screen.getByText(/authoring and builds are unavailable/),
    ).toBeDefined();
    const build = screen.getByRole<HTMLButtonElement>("button", {
      name: "Build",
    });
    expect(build.disabled).toBe(true);
  });
});
