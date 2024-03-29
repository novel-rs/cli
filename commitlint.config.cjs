const Configuration = {
  extends: ["@commitlint/config-angular"],
  rules: {
    "type-enum": [
      2,
      "always",
      ["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore"],
    ],
  },
};

module.exports = Configuration;
