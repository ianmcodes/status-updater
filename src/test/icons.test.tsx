import { render } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import {
  Radio,
  Server,
  Key,
  Tag,
  Check,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  Sliders,
  Eye,
  EyeOff,
  Save,
  FileKey,
  Layers,
  Minimize2,
  ListFilter,
} from "../icons";

describe("SVG Icon Components", () => {
  it("renders Radio icon with default and custom size and className", () => {
    const { container: c1 } = render(<Radio />);
    const svg1 = c1.querySelector("svg");
    expect(svg1).toBeInTheDocument();
    expect(svg1).toHaveAttribute("width", "24");
    expect(svg1).toHaveAttribute("height", "24");

    const { container: c2 } = render(<Radio size={16} className="test-class" data-testid="radio-icon" />);
    const svg2 = c2.querySelector("svg");
    expect(svg2).toHaveAttribute("width", "16");
    expect(svg2).toHaveAttribute("height", "16");
    expect(svg2).toHaveClass("test-class");
  });

  it("renders all remaining icon components without crashing", () => {
    const icons = [
      Server,
      Key,
      Tag,
      Check,
      CheckCircle2,
      AlertCircle,
      RefreshCw,
      Sliders,
      Eye,
      EyeOff,
      Save,
      FileKey,
      Layers,
      Minimize2,
      ListFilter,
    ];

    icons.forEach((IconComponent) => {
      const { container } = render(<IconComponent size={18} />);
      const svg = container.querySelector("svg");
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveAttribute("width", "18");
      expect(svg).toHaveAttribute("height", "18");
      expect(svg).toHaveAttribute("stroke", "currentColor");
    });
  });
});
