import { useState, useEffect } from "react";

export function useTheme(): "dark" | "light" {
  const [theme, setTheme] = useState<"dark" | "light">(() =>
    document.documentElement.classList.contains("dark") ? "dark" : "light"
  );

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");

    const update = (e: MediaQueryListEvent | MediaQueryList) => {
      const next = e.matches ? "dark" : "light";
      setTheme(next);
      if (next === "dark") {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
    };

    mq.addEventListener("change", update);
    return () => mq.removeEventListener("change", update);
  }, []);

  return theme;
}
