import { tool, type ToolSet } from "ai";
import { z } from "zod";
import type { SkillDefinition } from "../types";

export function parseSkillIds(skillsJson: string | null | undefined): Set<string> {
  if (!skillsJson) {
    return new Set<string>();
  }

  try {
    const parsed = JSON.parse(skillsJson) as unknown;
    if (Array.isArray(parsed)) {
      return new Set(
        parsed.filter((entry): entry is string => typeof entry === "string")
      );
    }
  } catch {
    // Ignore malformed persisted content.
  }

  return new Set<string>();
}

export function createSkillTools(params: {
  allSkills: SkillDefinition[];
  enabledSkillIds: Set<string>;
  requestedSkillIds: Set<string>;
}): ToolSet {
  const tools: ToolSet = {};

  for (const skill of params.allSkills) {
    if (
      !params.enabledSkillIds.has(skill.id) ||
      !params.requestedSkillIds.has(skill.id)
    ) {
      continue;
    }

    tools[skill.id] = tool({
      description: `Execute skill "${skill.name}" for local assistant tasks.`,
      inputSchema: z.object({}).passthrough(),
      execute: async (input) => {
        return await skill.execute(input);
      }
    });
  }

  return tools;
}
