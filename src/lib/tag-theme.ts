const TAG_PALETTE = [
  {
    background: "rgba(56, 189, 248, 0.14)",
    border: "rgba(14, 165, 233, 0.28)",
    text: "#0f5f8f",
    accent: "#0ea5e9",
  },
  {
    background: "rgba(34, 197, 94, 0.14)",
    border: "rgba(22, 163, 74, 0.28)",
    text: "#166534",
    accent: "#16a34a",
  },
  {
    background: "rgba(251, 146, 60, 0.16)",
    border: "rgba(249, 115, 22, 0.3)",
    text: "#9a3412",
    accent: "#f97316",
  },
  {
    background: "rgba(244, 114, 182, 0.16)",
    border: "rgba(236, 72, 153, 0.28)",
    text: "#9d174d",
    accent: "#ec4899",
  },
  {
    background: "rgba(168, 85, 247, 0.16)",
    border: "rgba(147, 51, 234, 0.28)",
    text: "#6b21a8",
    accent: "#9333ea",
  },
  {
    background: "rgba(250, 204, 21, 0.18)",
    border: "rgba(234, 179, 8, 0.3)",
    text: "#854d0e",
    accent: "#eab308",
  },
];

function hashTag(value: string) {
  let hash = 0;

  for (const character of value) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }

  return hash;
}

export function getTagTheme(tagName: string) {
  return TAG_PALETTE[hashTag(tagName) % TAG_PALETTE.length];
}
