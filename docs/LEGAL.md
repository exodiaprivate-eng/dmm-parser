# Legal & Distribution

Consolidated legal/distribution materials for dmm-parser, DMM, and the
broader Crimson Desert Modding ecosystem.

> **Last consolidated:** 2026-05-10. Merged from `LICENSE_DRAFT_v1.md`,
> `ENFORCEMENT_PLAYBOOK.md`, `NEXUSMODS_KIT.md`. The original three docs
> are deleted; their content lives in the sections below verbatim.

## Contents

- [License Draft (CDMTL v1.0)](#license-draft-cdmtl-v10)
- [Enforcement Playbook](#enforcement-playbook)
- [Nexus Mods Distribution Kit](#nexus-mods-distribution-kit)

---

## License Draft (CDMTL v1.0)

# CRIMSON DESERT MODDING TOOLS LICENSE v1.0 (DRAFT)
## A Modified Copyleft License Based on GNU General Public License v3.0

**Copyright (C) 2026 RicePaddySoftware. All Rights Reserved.**

> **STATUS: DRAFT — NOT YET LEGALLY EFFECTIVE.**
> This document is a working draft for review. It is NOT yet the operative license for any RicePaddySoftware project. See "Notes to Reviewer" at the bottom of this file for legal flags that must be resolved before adoption.

---

## 0. PREAMBLE

This License ("CDMTL v1.0") is a derivative work modeled on the GNU General Public License version 3.0 ("GPL v3"). It retains the copyleft principles of GPL v3 — namely, that modifications and redistributions must remain under this same license and that source must be made available — while adding scope restrictions specific to the Crimson Desert modding ecosystem.

**This License is NOT GPL v3 and is NOT compatible with GPL v3, MIT, Apache 2.0, or other OSI-approved licenses.** Code released under those licenses cannot be combined into Covered Work, and Covered Work cannot be relicensed under those terms without permission from RicePaddySoftware.

The intent of this License is to:

1. Permit free use, modification, and redistribution within the Authorized Software Suite.
2. Require that derivative works remain open-source and licensed under these same terms.
3. Prevent unauthorized commercial exploitation of the Field JSON v3.1 / Multi-Target Field Patching specification and its reference implementations.
4. Protect the canonical specifications and trademarks of RicePaddySoftware.
5. Establish enforceable copyright, trademark, and Copyright Management Information (DMCA §1202) protections that platform operators — including NexusMods, ModDB, GitHub, GameBanana, CurseForge, Steam Workshop, and similar mod-distribution services — can rely upon when evaluating takedown notices and trademark complaints concerning the Covered Work.

---

## 1. DEFINITIONS

**"Covered Work"** means the software and specifications licensed under this License, including any derivative works thereof.

**"Authorized Software Suite"** means the following software products, in their current and future versions, developed under the RicePaddySoftware umbrella:

  (a) **DMM** — Definitive Mod Manager (Tauri/Rust desktop application).
  (b) **SWISS Mod Manager** — successor or alternative mod manager developed by RicePaddySoftware.
  (c) **SWISS Save Editor** — Crimson Desert save-game editor.
  (d) **CrimsonGameMods Stacker** — mod creation and stacking tool.
  (e) **dmm-parser** — Rust library and Python bindings for Crimson Desert archive formats.
  (f) **Field JSON v3.1 specification** — and its reference implementations.
  (g) **Any other software** explicitly designated in writing as part of the Authorized Software Suite by RicePaddySoftware.

**"Field JSON Specification"** means the Field JSON v3, v3.1, or any subsequent versioned format authored by RicePaddySoftware for describing field-level modifications to Crimson Desert game archives.

**"Independent Tool"** means software that is NOT part of the Authorized Software Suite and has NOT been granted a written integration license.

**"Licensed Field"** means the field of modding the Crimson Desert video game (developed by Pearl Abyss) and reading, writing, or transforming its archive formats (PAPGT, PAMT, PAZ, PABGB, and successors).

**"You"** means the licensee — any individual or entity exercising rights under this License.

---

## 2. GRANT OF PERMISSIONS

Subject to the conditions and restrictions below, RicePaddySoftware grants You a worldwide, royalty-free, non-exclusive license to:

1. **Use** the Covered Work within the Licensed Field for personal, educational, or research purposes.
2. **Modify** the Covered Work, provided modifications are licensed under this same License.
3. **Redistribute** the Covered Work in source or binary form within the Licensed Field, provided this License accompanies the distribution and all conditions in Section 4 are met.
4. **Fork** the source code for development purposes, provided derivative public distributions remain under this License.

---

## 3. CONDITIONS (COPYLEFT REQUIREMENTS)

All redistributions of the Covered Work, in source or binary form, must:

1. **Retain notices** — all copyright notices, this License text, and any trademark notices in the source.
2. **Provide attribution** — clearly credit "RicePaddySoftware" and link to the canonical repository.
3. **State modifications** — clearly identify any changes made to the Covered Work, with dates.
4. **Disclose source** — when distributing binaries, the corresponding source code (including modifications) must be made available under this License at no charge or via a written offer valid for at least three years.
5. **Preserve license** — derivative works must be licensed under this License in its entirety. No additional restrictions and no relicensing under more permissive terms are allowed.
6. **No endorsement claims** — must not imply endorsement by RicePaddySoftware without prior written permission.

---

## 4. ADDITIONAL RESTRICTIONS (Beyond GPL v3)

The following clauses ADD restrictions beyond GPL v3 and are the reason this License is NOT GPL v3 compatible:

### 4.1 Authorized Suite Requirement

Software that **consumes, produces, parses, generates, or otherwise integrates** the Field JSON Specification for the purpose of modifying Crimson Desert game archives **must** satisfy at least one of the following:

  (a) Be part of the Authorized Software Suite, OR
  (b) Have prior written permission from RicePaddySoftware, OR
  (c) Be a personal-use tool not distributed to other users (single-user, non-public).

### 4.2 No Independent Tool Integration

The Covered Work, including binary or source-form copies of the dmm-parser library, the Field JSON Specification, or reference implementations thereof, may NOT be embedded, statically linked, dynamically linked, vendored, or otherwise integrated into any Independent Tool without a written integration license from RicePaddySoftware.

### 4.3 No Derivative Specifications

You may not publish, distribute, or promote derivative or competing specifications that present themselves as compatible extensions, successors, or replacements for the Field JSON Specification without explicit written designation by RicePaddySoftware. This clause exists to protect the canonical specification from fragmentation.

This restriction does not prohibit:
  - Academic discussion or comparison of the specification.
  - Bug reports, security advisories, or interoperability documentation.
  - Independent specifications targeting different file formats or use cases.

### 4.4 No Commercial Exploitation by Independent Tools

Independent Tools may not commercially distribute, monetize, or sell access to functionality derived from the Covered Work or the Field JSON Specification without a commercial license from RicePaddySoftware.

This restriction does not affect:
  - Sale of unmodified Authorized Software Suite components by RicePaddySoftware.
  - Donations, tips, or voluntary contributions to mod authors using the Authorized Software Suite.
  - Sale of mods (the JSON files themselves) made BY end users, provided no fee is charged for the tools used to create them outside the Authorized Software Suite.

### 4.5 Trademark and Naming

The following are trademarks of RicePaddySoftware:

  - "Field JSON v3.1"
  - "Multi-Target Field Patching"
  - "DMM" / "Definitive Mod Manager"
  - "SWISS" / "SWISS Suite"
  - "RicePaddySoftware"
  - "CrimsonGameMods"

Use of these marks in derivative works, tools, or documentation requires written permission. Compatibility statements ("works with DMM v3.1") are permitted descriptive uses and do NOT require permission.

Use of confusingly similar names — including but not limited to abbreviations, acronyms, transliterations, or stylistic variants of the above marks (such as "DUMM", "CDUMM", "DMM2", "S.W.I.S.S.", "FieldJSON3", "CrimsonModManager", "DefinitiveMM", or any name that combines "Crimson", "Desert", "Definitive", or "SWISS" with mod-management or save-editing terminology) — is prohibited where such use creates a likelihood of confusion in the modding community as to source, sponsorship, or affiliation with RicePaddySoftware.

### 4.6 Copyright Management Information (DMCA §1202)

The Covered Work contains copyright management information ("CMI") within the meaning of 17 U.S.C. § 1202, including:

  (a) The identification of RicePaddySoftware as author and copyright owner.
  (b) The text of this License.
  (c) Trademark notices for "Field JSON v3.1", "DMM", "SWISS", and related marks.
  (d) Per-file copyright headers in source code.
  (e) Version numbers, dates of authorship, and origin URLs (canonical repository links).

**You may not intentionally remove, alter, or falsify any CMI** in the Covered Work, nor distribute the Covered Work knowing that CMI has been removed or altered. Doing so constitutes a separate violation of 17 U.S.C. § 1202 and exposes the violator to statutory damages of $2,500 to $25,000 per violation, plus attorneys' fees, independently of any other remedies under this License.

This clause is intended to be enforced via DMCA takedown notice on any platform hosting unauthorized copies of the Covered Work, including but not limited to mod distribution platforms.

### 4.7 Distribution Platform Restrictions

The Covered Work, in source or binary form, including any modifications or derivative works, **may only be hosted, distributed, mirrored, or made publicly available on the following Authorized Distribution Channels**:

  (a) Repositories under the GitHub organizations or accounts of RicePaddySoftware, including https://github.com/exodiaprivate-eng, https://github.com/NattKh, and any successor accounts officially designated by RicePaddySoftware.
  (b) NexusMods pages owned and operated by RicePaddySoftware or its designated agents.
  (c) The official RicePaddySoftware website (when established) and its associated download CDN.
  (d) Any other distribution channel explicitly authorized in writing by RicePaddySoftware.

**Hosting, mirroring, or redistributing the Covered Work or derivative works on third-party platforms is expressly prohibited without written permission.** Prohibited platforms include, but are not limited to:

  - NexusMods, ModDB, GameBanana, CurseForge, Steam Workshop, ModNexus, Mod.io, and any successor or competitor mod distribution service, when the upload is by anyone other than RicePaddySoftware or an authorized agent.
  - GitHub repositories, GitLab, Bitbucket, Codeberg, or any source-code hosting platform, when the repository is not part of the Authorized Distribution Channels.
  - File-sharing services (MediaFire, Mega, Google Drive, Dropbox, etc.) when used for public redistribution rather than personal backup.
  - Discord servers, Reddit posts, forum attachments, or any chat/social platform when the posting constitutes public redistribution.

This restriction is intended to give RicePaddySoftware a clear basis for issuing takedown notices to platform operators when unauthorized copies are detected.

### 4.8 No Forking for Public Redistribution

While You may create personal forks of the Covered Work for development purposes (per Section 2.4), publishing such forks for public consumption — including making them available on NexusMods, third-party GitHub repositories, or other distribution channels not listed in Section 4.7 — requires prior written authorization from RicePaddySoftware.

This restriction does not affect:
  - Pull requests submitted to the official RicePaddySoftware repositories.
  - Private development forks not shared publicly.
  - Forks created solely for the purpose of submitting bug reports or security advisories.

### 4.9 No Competing Implementation by Licensees

By accepting this License — including by cloning, downloading, building, executing, viewing, reading, indexing, scraping, training upon, or otherwise accessing the Covered Work or the Field JSON Specification documentation, whether directly by Yourself or through an AI assistant, large language model, autonomous coding agent, code-analysis tool, web scraper, search indexer, or any other automated system acting on Your behalf or at Your direction — You agree that, **for a period of three (3) years from the date of Your first such access**, You will not directly or indirectly:

  (a) Develop, publish, or distribute a software tool that performs substantially the same functions as the Authorized Software Suite (i.e., a Crimson Desert mod manager, save editor, or mod-stacking tool) AND that consumes, produces, or interoperates with the Field JSON Specification or any successor format.

  (b) Use knowledge of the Covered Work's internal architecture, methodology, application logic, or mod-mounting/overlay mechanisms — including but not limited to: the runtime injection approach, the field-level patching dispatch system, the typed-table apply mechanism, the PAPGT/PAMT/PAZ rewriting strategy, the trie-based archive lookup, or the multi-target field intent resolution algorithm — to develop a competing tool.

  (c) Reverse engineer the Covered Work, decompile binary distributions, or analyze source code for the purpose of creating a competing implementation, except as expressly permitted by 17 U.S.C. § 1201(f) (interoperability with independently-created programs that do not compete with the Covered Work) or equivalent statutory provisions.

This clause exists because U.S. copyright law (17 U.S.C. § 102(b)) does not extend protection to methods, processes, or systems — only to specific code expression. This License therefore relies on contract law to protect the functional architecture and methodology of the Covered Work against parties who have voluntarily accepted these terms by accessing the Covered Work.

**Permitted Activities** (do not violate this clause):

  1. **Independent prior creation** — Tools that the developer can demonstrate were independently developed without ANY exposure to the Covered Work, the Field JSON Specification, or RicePaddySoftware documentation. The burden of proving independent creation rests with the developer.

  2. **Interoperability without competition** — Tools that read Field JSON v3.1 mod files for archival, conversion, or display purposes, but that DO NOT apply mods to live Crimson Desert game files.

  3. **Academic study and publication** — Research papers, tutorials, blog posts analyzing the Covered Work, provided no competing tool results.

  4. **Contributions to the Authorized Software Suite** — Pull requests, plugins, and extensions accepted into the canonical RicePaddySoftware repositories.

  5. **Expiration** — After the three-year period elapses, this clause no longer restricts the licensee.

**Enforcement:** Violation of this clause constitutes a material breach of this License, terminating all rights under Section 7 and exposing the violator to contract-law remedies including injunctive relief, disgorgement of profits, and damages. RicePaddySoftware may also identify the violator publicly and request that platform operators remove the violating tool on grounds of License breach.

### 4.10 Acceptance by Access (Including AI-Mediated Access)

**Acceptance of this License is triggered by ANY access to the Covered Work**, regardless of the means, duration, or intermediary involved. "Access" includes, but is not limited to:

  (a) Cloning, forking, or downloading any RicePaddySoftware repository.
  (b) Reading source code via the GitHub web interface, GitLab, Sourcegraph, Sourcehut, or any code-browsing service.
  (c) Downloading binary releases, installers, or packaged distributions.
  (d) Reading the Field JSON Specification document, README files, or related documentation.
  (e) Executing the Covered Work or any binary built from it.
  (f) Inspecting compiled binaries via decompilation, disassembly, static analysis, or dynamic analysis tools.
  (g) Reading the Covered Work via web archives (Wayback Machine, archive.today, Google Cache, GitHub Archive Program) or any successor archival service.
  (h) Receiving or viewing copies distributed by third parties, regardless of whether such distribution was authorized.

#### 4.10.1 Acceptance via AI Assistants and Automated Agents

Access to the Covered Work via an AI assistant, large language model (LLM), code-analysis agent, autonomous coding tool, web scraper, search engine indexer, retrieval-augmented-generation (RAG) system, or any other automated system acting on Your behalf or at Your direction **constitutes Your acceptance of this License**. You cannot escape the obligations of this License by interposing an AI, agent, or other automated intermediary between Yourself and the Covered Work.

This includes, but is not limited to:

  (a) Prompting an AI assistant — such as Claude (Anthropic), ChatGPT (OpenAI), Gemini (Google), Cursor, GitHub Copilot, Sourcegraph Cody, Aider, Cline, Devin, Windsurf, Replit Agent, or any present or future AI coding tool — to read, summarize, analyze, refactor, translate, port, or replicate the Covered Work.
  (b) Using an autonomous coding agent or AI-driven development environment to clone, decompile, parse, or process RicePaddySoftware repositories, binaries, or documentation.
  (c) Using a web-scraping service, code-indexing service, or AI training pipeline to retrieve the Covered Work for analysis or model training.
  (d) Receiving AI-generated explanations, summaries, pseudocode, architectural diagrams, or refactored code derived from the Covered Work.
  (e) Using AI-generated code that incorporates, paraphrases, structurally mirrors, or is derived from the Covered Work, regardless of whether You personally observed the source code from which it was derived.
  (f) Asking an AI to "build a tool similar to DMM" or "implement a Crimson Desert mod manager" if the AI's response is informed (whether at training time or inference time) by the Covered Work.

#### 4.10.2 No AI Training Without Authorization

The Covered Work may not be used as training data, fine-tuning data, retrieval-augmented-generation (RAG) source material, in-context-learning examples, or any other form of input to an AI model, agent, or automated system intended to:

  (a) Generate competing implementations of the Authorized Software Suite (per Section 4.9).
  (b) Extract or replicate the methodology, architecture, or application logic of the Covered Work (per Section 4.9).
  (c) Produce derivative documentation of the Field JSON Specification without attribution to RicePaddySoftware.
  (d) Generate code, documentation, or analysis that would, if produced by a human licensee, violate any other clause of this License.

Use of the Covered Work as input to AI tools for purposes consistent with the Authorized Software Suite ecosystem (e.g., a Claude-assisted developer working on a RicePaddySoftware-approved contribution to DMM) is permitted, provided such use complies with all other clauses of this License.

#### 4.10.3 Imputation of Knowledge

Knowledge, methodology, architecture, application logic, or implementation details extracted from the Covered Work via AI assistants, automated agents, or any other intermediary tool **are imputed to You** — the human user who directed, prompted, or otherwise caused the access — for purposes of Section 4.9 (No Competing Implementation by Licensees).

You cannot claim "the AI read it, not me" as a defense to building a competing tool. Under principles of agency law, an automated system acting on Your behalf is legally equivalent to Your own action for purposes of License acceptance and breach analysis.

#### 4.10.4 Reasonable Notice

This License is published prominently:

  (a) As `LICENSE.txt` (or equivalent) at the root of every RicePaddySoftware repository.
  (b) As a copyright header at the top of every source file in the Covered Work.
  (c) In the README of every RicePaddySoftware repository.
  (d) On the NexusMods page (and other Authorized Distribution Channels) for every distributed binary.
  (e) At the canonical URL https://github.com/exodiaprivate-eng/dmm-parser/blob/main/LICENSE.txt (or successor URL designated by RicePaddySoftware).

By accessing the Covered Work through any of these channels — or any unauthorized copy that retains the original notices — You acknowledge constructive notice of these terms. Stripping these notices does not eliminate Your obligations under this License; it instead constitutes an additional violation under Section 4.6 (Copyright Management Information / DMCA §1202).

---

## 5. PERMITTED EXCEPTIONS

The following uses are explicitly permitted without separate authorization, even where Sections 4.1–4.4 would otherwise restrict them:

1. **Personal use** — A single individual using a personal tool for their own gameplay, with no public distribution.
2. **Educational use** — Academic study, classroom instruction, and research on the file formats and specifications.
3. **Security research** — Vulnerability discovery, responsible disclosure, and defensive security analysis.
4. **Interoperability with non-CD tooling** — Translation FROM Field JSON Specification TO unrelated archival or interchange formats, where the destination is NOT used to modify Crimson Desert game archives.
5. **Bug reports and feedback** — Submission of issues, pull requests, and constructive criticism.
6. **Reverse engineering for compatibility** — As required and permitted by applicable copyright law (e.g., 17 U.S.C. § 1201(f) in the United States, equivalent EU directives, etc.).

---

## 6. CONTRIBUTIONS

Contributions submitted to RicePaddySoftware repositories are governed by a separate Contributor License Agreement (CLA) [TO BE DRAFTED]. By submitting a contribution, You agree to license it under this License and grant RicePaddySoftware the right to relicense future versions if needed.

If no CLA is in place at the time of contribution, the contribution is presumed to be licensed under CDMTL v1.0 with the right of RicePaddySoftware to incorporate it into the Authorized Software Suite.

---

## 7. TERMINATION

Your rights under this License terminate automatically if You materially violate any of its terms.

You may regain Your rights by:

1. Ceasing the violating activity within thirty (30) days of receiving written notice, AND
2. Either curing the violation OR receiving written reinstatement from RicePaddySoftware.

Termination does not affect:
  - The rights of downstream end users who received copies of the Covered Work in compliance with this License before Your termination.
  - The validity of mods created by end users using compliant Authorized Software Suite tools.

---

## 8. ENFORCEMENT MECHANISM

RicePaddySoftware reserves all rights and remedies available under applicable copyright, trademark, and contract law to enforce this License. Without limiting the foregoing, RicePaddySoftware specifically intends to enforce violations through the following mechanisms:

### 8.1 DMCA Takedown Notices

For unauthorized hosting or distribution of the Covered Work on third-party platforms (including but not limited to NexusMods, ModDB, GameBanana, GitHub, GitLab, CurseForge, Steam Workshop, Mod.io, and similar services), RicePaddySoftware will issue takedown notices under 17 U.S.C. § 512 (Digital Millennium Copyright Act) directly to the platform operator. By accepting this License, You acknowledge that:

  (a) Hosting or redistributing the Covered Work outside the Authorized Distribution Channels (Section 4.7) constitutes copyright infringement actionable under DMCA.
  (b) Removing or altering Copyright Management Information (Section 4.6) is independently actionable under 17 U.S.C. § 1202 with statutory damages of $2,500 to $25,000 per violation.
  (c) Platform operators are entitled to rely on RicePaddySoftware's good-faith DMCA notices in removing infringing content.

### 8.2 Trademark Enforcement

For unauthorized use of RicePaddySoftware trademarks (Section 4.5), including confusingly similar names targeting the Crimson Desert modding community, RicePaddySoftware will pursue:

  (a) Trademark complaints to the hosting platform under its trademark policy.
  (b) UDRP (Uniform Domain-Name Dispute-Resolution Policy) proceedings for domain-name infringement.
  (c) Cease-and-desist letters and trademark infringement litigation under the Lanham Act (15 U.S.C. § 1051 et seq.) or equivalent foreign trademark law.

### 8.3 License Violation Remedies

For violations of License terms not otherwise covered by copyright or trademark enforcement (e.g., breach of the Authorized Suite Requirement in Section 4.1), RicePaddySoftware reserves the right to:

  (a) Terminate the violator's License grant under Section 7.
  (b) Pursue contract-law remedies, including specific performance and damages.
  (c) Seek injunctive relief to prevent ongoing violations.
  (d) Publicly identify violators and excluded forks via official RicePaddySoftware communication channels.

### 8.4 No Waiver

Failure of RicePaddySoftware to enforce any provision of this License at any time shall not be construed as a waiver of the right to enforce that provision later. Selective enforcement does not create a defense or estoppel for any violator.

### 8.5 Cooperation with Platform Operators

This License is intended to be self-executing for the purposes of platform-mediated enforcement. RicePaddySoftware grants platform operators (NexusMods, ModDB, GitHub, etc.) the right to rely on this License text as a basis for evaluating takedown requests, DMCA notices, and trademark complaints concerning the Covered Work.

---

## 9. WARRANTY DISCLAIMER

THE COVERED WORK IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NONINFRINGEMENT, AND TITLE.

IN NO EVENT SHALL RICEPADDYSOFTWARE OR ANY CONTRIBUTOR BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THE COVERED WORK, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

---

## 10. RELATIONSHIP TO THE GAME

The Covered Work is a third-party modding toolkit. It is **not** affiliated with, endorsed by, or sponsored by Pearl Abyss, the developers of Crimson Desert, or any related entity. End users assume all responsibility for compliance with the game's End User License Agreement and applicable laws when using the Covered Work to modify game files.

---

## 11. JURISDICTION AND DISPUTE RESOLUTION

This License shall be governed by and construed in accordance with the laws of **[JURISDICTION TO BE DETERMINED — typically the country/state where RicePaddySoftware is incorporated]**, without regard to conflict-of-laws principles.

Any dispute arising out of or relating to this License shall be resolved by **[binding arbitration / courts of competent jurisdiction in JURISDICTION — to be decided]**.

---

## 12. ACKNOWLEDGMENT

By using, modifying, distributing, accessing, reading, viewing, downloading, executing, or otherwise interacting with the Covered Work — whether directly by Yourself or through an AI assistant, automated agent, code-analysis tool, web scraper, or any other intermediary acting on Your behalf or at Your direction — You acknowledge that You have read (or had constructive notice of) this License, understand its terms, and agree to be bound by them. If You do not agree to these terms, You must not access, use, or direct any agent (human or automated) to access or use the Covered Work.

---

## 13. SEVERABILITY

If any provision of this License is found unenforceable in any jurisdiction, the remaining provisions shall continue in full force and effect, and the unenforceable provision shall be interpreted to most closely match the original intent in a manner that is enforceable.

---

# NOTES TO REVIEWER (Not Part of the License)

These are concerns and open questions you should resolve **before** adopting this license. I am not a lawyer; this draft is a starting point, not legal advice. **Consult a software licensing attorney before publishing.**

## Critical Legal Realities

1. **You cannot call this "GPL v3" or "GNU GPL"** — those names are protected. The FSF's GPL v3 expressly forbids modification and renaming. The most you can say is "modeled on" or "based on the structure of" GPL v3. I named this draft "CDMTL v1.0" (Crimson Desert Modding Tools License) instead.

2. **This is NOT open source by OSI definition.** The Open Source Initiative will not certify any license that restricts who can use software or for what purpose. You will be operating in "source-available" / "shared-source" territory — closer to BUSL (Business Source License) or Elastic License v2 than GPL.

3. **License incompatibility is bidirectional.** Once you adopt CDMTL:
   - You cannot pull in code from MIT/Apache/GPL projects without violating their terms (their licenses don't allow your additional restrictions).
   - GPL/MIT/Apache projects cannot pull from your code.
   - This includes most Rust crates on crates.io (most are MIT/Apache-2.0 dual licensed).
   - **dmm-parser currently uses PyO3 (Apache-2.0/MIT)** — this is OK because Apache/MIT permit redistribution under more restrictive licenses for compiled output. But any AGPL or LGPL dependency would be a hard block.

4. **Existing GPL'd or MIT'd contributions cannot be retroactively relicensed.** If anyone has contributed code to dmm-parser, DMM, or CrimsonGameMods under their existing license, you need their explicit consent to relicense under CDMTL. Without a CLA in place from day 1, you have a relicensing problem.

5. **"RicePaddySoftware" must be a real legal entity** for many of these clauses to be enforceable in court. If it's currently just a name you're using, register it (LLC, sole proprietorship, etc.) before publishing the license.

6. **Trademark protection requires registration** — claiming "Field JSON v3.1™" only has bite if you've actually filed a trademark in your jurisdiction. Until then, you have common-law trademark rights at best, which are weak.

## Open Drafting Questions

I left **[BRACKETED PLACEHOLDERS]** in the draft. You need to decide:

1. **Jurisdiction (Section 10)** — Where are you incorporated / where do you live? US state, UK, EU country, etc. This determines which laws apply.

2. **Dispute resolution** — Arbitration (cheaper, private, harder to appeal) vs. courts (public, slower, more expensive).

3. **Contributor License Agreement** — Section 6 references a CLA that doesn't exist yet. Do you want one? (Highly recommended for any project accepting outside PRs.)

4. **Patent grant clause** — GPL v3 has an explicit patent grant. I omitted it from this draft because it's complex; we should add one if you want patent protection.

5. **"Future versions" auto-upgrade** — GPL v3 has language allowing licensees to choose any future version. Do you want CDMTL v2 to automatically apply, or should each version stand alone?

6. **End-user-game-modification disclaimer (Section 9)** — Pearl Abyss' EULA might forbid modding entirely. Section 9 disclaims your responsibility, but you may want stronger language.

## Things I Did NOT Address

- **GDPR / privacy** — if any tool collects user data, separate privacy policy needed.
- **Export controls** — cryptography in dmm-parser (ChaCha20) may trigger US export controls or equivalent.
- **DMCA takedown response** — if Pearl Abyss tries to take down the tools, what's your response plan?
- **Sub-licensing structure** — can you license CDMTL components to a partner separately? Currently no, intentionally.

## Recommended Next Steps

1. Review the draft and tell me which clauses to tighten, loosen, or remove.
2. Decide jurisdiction so I can fill in Section 10.
3. Decide whether to retain or remove Section 4.4 (commercial exploitation) — this is the harshest clause and may discourage community contributions.
4. Consider whether to add a "free for non-commercial Crimson Desert mod authors" carve-out — this would let independent modders use SWISS-derived code in their own personal mod tools.
5. Get this in front of an actual licensing attorney before publishing. Cost: typically $500–$2,000 USD for a license review by a software-IP-focused lawyer. Worth it.

---

*End of Draft.*

---

## Enforcement Playbook

# RicePaddySoftware Enforcement Playbook

**Companion document to `LICENSE_DRAFT_v1.md` (CDMTL v1.0).**

This playbook explains, in plain English, how to actually use the license to get unauthorized copies of DMM, SWISS, dmm-parser, or Field JSON v3.1 implementations taken down from NexusMods, GitHub, and other platforms.

> **Disclaimer:** I am not a lawyer. This playbook is operational guidance based on publicly available platform policies and US copyright law. Consult a licensed attorney before sending formal legal notices in disputed cases.

---

## The Honest Truth About What NexusMods Will Enforce

NexusMods is a US-based content host. Their takedown policy is shaped by the Digital Millennium Copyright Act (DMCA) safe-harbor provisions in 17 U.S.C. § 512. They are NOT going to evaluate the merits of your custom license terms — they will only act on claims they can clearly verify.

### What WORKS (NexusMods will take it down)

| Claim Type | Why It Works | Strength |
|---|---|---|
| **Direct code copying** | Pure DMCA copyright. They diff your code vs theirs. | Very strong |
| **Asset/binary theft** | Your compiled .exe, icons, screenshots reused | Very strong |
| **Copyright Management Information removal** | DMCA §1202, statutory damages | Very strong |
| **Trademark infringement** (registered marks) | Lanham Act claim | Strong |
| **Confusingly similar product names** | Trademark dilution | Moderate (better with registered mark) |
| **Tool re-uploaded without your consent** | Mirroring = redistribution = copyright | Strong |
| **Stripped attribution / removed your name** | DMCA §1202 + bad faith | Very strong |

### What DOES NOT WORK (NexusMods will tell you to sue in court)

| Claim Type | Why It Fails |
|---|---|
| **"They violated my license restrictions"** | Contract dispute, not copyright; not actionable via DMCA |
| **"They reverse-engineered my tool"** | Clean-room RE is generally legal; fair use defenses apply |
| **"They consume my JSON format"** | File formats themselves are not copyrightable (idea/expression dichotomy) |
| **"My license forbids this use case"** | Use restrictions are contract terms; NexusMods doesn't enforce contracts |
| **"They competed with me commercially"** | Antitrust says competition is fine; not their problem |
| **"Their tool is similar to mine"** | Functional similarity is not copyright infringement |

**Key insight:** Your license's "Authorized Suite" restriction is a CONTRACT term. It's enforceable in court but NOT via NexusMods takedown. The takedown ammunition comes from copyright (code copying), trademark (name infringement), and CMI removal (§1202). The license SUPPORTS those claims by establishing what's protected and how it's marked.

---

## Five Things You Must Do To Make The License Enforceable

The license document is necessary but not sufficient. To actually win takedowns, you need supporting evidence and registrations.

### 1. Add Per-File Copyright Headers (do this first, takes 2 hours)

Every source file should start with a header like this (Rust example):

```rust
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
//
// Licensed under the Crimson Desert Modding Tools License v1.0 (CDMTL v1.0).
// See LICENSE.txt at the repository root for full terms.
// Canonical URL: https://github.com/exodiaprivate-eng/dmm-parser
//
// NOTICE OF ACCEPTANCE BY ACCESS:
// Reading, viewing, downloading, cloning, decompiling, or otherwise
// accessing this file — whether directly by a human or via an AI
// assistant, autonomous agent, code-analysis tool, web scraper, or
// any other automated system — constitutes acceptance of CDMTL v1.0.
// See § 4.10 for AI-mediated access terms.
//
// NO COMPETING IMPLEMENTATION:
// By accessing this file You agree not to use its content, methodology,
// or architecture to develop a competing Crimson Desert mod manager
// for three (3) years. See § 4.9.
//
// CMI NOTICE (17 U.S.C. § 1202):
// This header constitutes Copyright Management Information.
// Removal or alteration triggers statutory damages of $2,500-$25,000
// per violation, plus attorneys' fees, independently of any
// underlying copyright infringement.
```

For Python files:
```python
# SPDX-License-Identifier: LicenseRef-CDMTL-1.0
# Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
# Licensed under CDMTL v1.0 — see LICENSE.txt
# https://github.com/exodiaprivate-eng/dmm-parser
#
# Reading this file (directly or via AI/agent) constitutes acceptance
# of CDMTL v1.0, including § 4.9 (No Competing Implementation) and
# § 4.10 (AI-Mediated Access). Removal of this notice violates 17 U.S.C. § 1202.
```

For TypeScript/JavaScript:
```typescript
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 — see LICENSE.txt
// https://github.com/exodiaprivate-eng/DMM-BETA
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 § 4.9 (No Competing Implementation) and § 4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. § 1202.
```

**Why this matters legally:**

1. **CMI teeth (§1202)** — Per-file CMI gives you stacked statutory damages. 100+ Rust files × $2,500 minimum = $250,000+ in damages even before proving the underlying copy.

2. **"Acceptance by access" enforceability** — Browsewrap-style terms (acceptance triggered by access) are only enforceable when there is **reasonable notice**. Putting the notice IN every file means:
   - Anyone who opens the file in an editor sees it
   - Anyone who clones the repo sees it on the first file they read
   - AI assistants ingesting the file include the notice in their context
   - Courts have a clean record of "the user could not have read the code without seeing these terms"

3. **AI-mediated access binding** — Under principles of agency law, what your AI does on your behalf is legally your action. Putting "AI access = your acceptance" in the header strengthens this argument by establishing both the user AND the AI agent are on notice of the terms.

4. **Defeats the "I never agreed" defense** — Without per-file headers, a violator can argue they cloned the repo to look at one file, never opened LICENSE.txt, and therefore never knew about the terms. With per-file headers, this defense fails — every file they touched contained the terms.

### 2. Establish Canonical Distribution Channels (do this now, takes 30 min)

Document publicly which URLs are AUTHORIZED:

```
Authorized Distribution Channels for RicePaddySoftware Tools:

- Source: https://github.com/exodiaprivate-eng/dmm-parser
- Source: https://github.com/exodiaprivate-eng/DMM-BETA
- Source: https://github.com/NattKh/CRIMSON-DESERT-SAVE-EDITOR-AND-GAME-MODS
- NexusMods: [your nexusmods page URL when published]
- Releases: https://github.com/exodiaprivate-eng/<repo>/releases

Any copy of these tools found OUTSIDE these URLs is unauthorized
and subject to DMCA takedown.
```

Put this in:
- `README.md` of each repo
- Your NexusMods mod page description
- A pinned issue or wiki page on your main repo

**Why this matters:** When you file a DMCA notice, you need to prove the copy is unauthorized. "It's not on my list of authorized channels" is a clean evidentiary claim.

### 3. Register Your Copyright with the US Copyright Office (one-time, $45-65)

US copyright is automatic on creation, BUT:
- Registered copyright lets you claim **statutory damages** ($750–$30,000 per work, up to $150,000 for willful infringement)
- Registered copyright lets you recover **attorneys' fees**
- Without registration, you can only sue for actual damages (which are usually $0 for free software)

**How:**
1. Go to https://www.copyright.gov
2. Register the dmm-parser source code as a "literary work, computer program"
3. Register the Field JSON v3.1 spec as a separate "literary work"
4. Cost: $45 single-author, $65 organization
5. Takes 3-6 months to issue but is retroactively effective from filing date

**Why this matters:** Without registration, your DMCA still works, but if the violator fights back, you have weak monetary leverage in court. Registration multiplies your settlement leverage 100x.

### 4. File for Trademark Protection (when you can afford it, $250-350 per mark)

Trademark protections in the license are mostly defensive without registration. To enforce:

**Priority 1 — file these:**
- "DMM" (or "Definitive Mod Manager") — for software
- "SWISS Suite" — for software
- "Field JSON v3.1" — could be hard, format names sometimes get rejected as descriptive

**Priority 2 — file if you have budget:**
- "RicePaddySoftware"
- "CrimsonGameMods"

**How:**
1. Search USPTO TESS database first (https://www.uspto.gov/trademarks/search) — make sure no one else owns these
2. File via USPTO TEAS Plus ($250) or hire a trademark attorney ($1,500-3,000 turnkey)
3. Use Class 9 (Computer software) for product marks
4. Can take 6-12 months for registration

**Why this matters:** Common-law trademark exists but is jurisdiction-limited and weak in court. Registered ® gives you nationwide rights, statutory damages for counterfeit, and customs/border enforcement.

### 5. Build An Evidence Trail (do this every release)

For every release of DMM, SWISS, dmm-parser:

- Tag the git commit with `v1.2.3` (timestamped, immutable)
- Publish a GitHub Release with the binary
- Post the release on your NexusMods page
- Save a SHA-256 hash of every distributed binary in your records
- Take a screenshot of the GitHub Release page (in case it gets deleted)

**Why this matters:** When filing DMCA, you need to prove **prior authorship**. "I have a git commit dated 2026-04-15 with SHA-256 `abc123...` matching the binary they uploaded on 2026-04-20" is irrefutable. Without this trail, the violator can claim independent creation.

---

## How To File a DMCA Takedown on NexusMods (Step-by-Step)

When you find unauthorized copies of DMM, SWISS, or v3.1 implementations on NexusMods:

### Step 1: Document the Violation

Before sending anything, capture:
- Full URL of the infringing mod page
- Screenshot of the page (in case they delete)
- Download the file (preserve evidence)
- Compare the binary to yours: `sha256sum violator.exe` vs `sha256sum your-release.exe`
- Diff any source code files visible
- Check if they removed your copyright headers (this is your §1202 claim)

### Step 2: Write the DMCA Notice

NexusMods accepts DMCA notices at: **dmca@nexusmods.com**

Standard DMCA notice template:

```
SUBJECT: DMCA Takedown Notice — Unauthorized Copy of [Tool Name]

To: NexusMods DMCA Agent

I am the copyright owner of [Tool Name], distributed under the
Crimson Desert Modding Tools License v1.0 by RicePaddySoftware.

1. IDENTIFICATION OF COPYRIGHTED WORK:
   Tool: [DMM / SWISS Save Editor / dmm-parser / etc.]
   Original distribution URL: https://github.com/exodiaprivate-eng/[repo]
   Copyright registration: [TX-12345-678 if registered, otherwise "common-law copyright"]
   First publication date: [date of first commit/release]

2. IDENTIFICATION OF INFRINGING MATERIAL:
   URL: https://www.nexusmods.com/crimsondesert/mods/[ID]
   Uploader: [username]
   Date posted: [date]

3. EVIDENCE OF INFRINGEMENT:
   The uploader has redistributed my Covered Work without authorization.
   Specifically:
   (a) [SHA-256 hashes match — describe]
   (b) [Copyright headers removed — describe, this triggers §1202]
   (c) [Distribution outside Authorized Channels per CDMTL v1.0 §4.7]
   (d) [Confusingly similar branding per CDMTL v1.0 §4.5]

4. GOOD FAITH STATEMENT:
   I have a good faith belief that use of the material in the
   manner complained of is not authorized by the copyright owner,
   its agent, or the law.

5. ACCURACY STATEMENT:
   The information in this notification is accurate, and under
   penalty of perjury, I am authorized to act on behalf of the
   copyright owner of the exclusive right that is allegedly infringed.

6. CONTACT INFORMATION:
   Name: [Your legal name OR "RicePaddySoftware Authorized Agent"]
   Email: [your email]
   Address: [your physical address — REQUIRED by DMCA]
   Phone: [your phone — REQUIRED by DMCA]

7. SIGNATURE:
   /s/ [Your name]
   Date: [today]

ATTACHMENTS:
- Screenshot of infringing page
- Download of infringing file (SHA-256: ...)
- Diff showing copied code (if applicable)
- Original copyright headers vs stripped version (§1202 evidence)
- Reference to LICENSE.txt at https://github.com/exodiaprivate-eng/[repo]/blob/main/LICENSE.txt
```

### Step 3: Send and Track

- Send to dmca@nexusmods.com
- Save the email + any auto-reply with case number
- Expected response time: 1-7 days
- NexusMods will either take the content down or forward your notice to the uploader for counter-notice

### Step 4: Handle Counter-Notice (if it happens)

If the uploader files a counter-notice claiming the takedown was wrongful:
- NexusMods is REQUIRED by DMCA to put the content back up after 10-14 business days **unless you file a lawsuit**
- This is the moment you actually need an attorney
- Most counter-notices are bluffs; many violators give up at this stage rather than face litigation
- If you don't litigate, the content stays up — but you can re-file if they violate again

### Step 5: Repeat Offender Tracking

NexusMods tracks repeat infringers. After 3+ DMCA strikes, they typically ban the uploader's account. Document each takedown so you can show a pattern.

---

## How To File a DMCA Takedown on GitHub (Different Process)

GitHub's DMCA process: https://github.com/contact/dmca

Key differences from NexusMods:
- GitHub publishes all DMCA notices publicly at https://github.com/github/dmca
- Allows specific-line takedowns rather than whole-repo
- Repeat-infringer policy applies to entire GitHub accounts
- Counter-notice puts content back in 10-14 days without litigation

Use the same notice template above, but submit via the GitHub web form.

---

## Trademark Complaints (Different from DMCA)

DMCA = copyright. For trademark issues (someone using "DMM2" or "CDUMM" for a tool that confuses users), use the platform's separate trademark complaint process:

**NexusMods trademark complaint:**
Email: legal@nexusmods.com (NOT dmca@)
Provide:
- Your trademark registration (or evidence of common-law mark use)
- The infringing use
- Why it creates confusion in the modding community
- Reference to CDMTL v1.0 §4.5 (Trademark and Naming clause)

**GitHub trademark complaint:**
https://docs.github.com/en/site-policy/content-removal-policies/github-trademark-policy

---

## Detecting AI-Mediated Copying

When CDUMM (or any competitor) is built with AI assistance, you have several signals to look for:

### Signature patterns of AI-generated code from your codebase

**1. Architectural fingerprints** — AI assistants tend to preserve architectural patterns from their input. If their tool has:
- The same module breakdown (binary/, crypto/, item_info/)
- The same naming conventions (`parse_table`, `serialize_table`, `apply_v3_for_target`)
- The same struct field ordering
- The same dispatch table with similar match arms

...that's strong evidence the AI was given dmm-parser as context.

**2. Comment style transfer** — AI assistants often preserve comment style and even verbatim comments from training/context data. If their codebase has:
- Comments matching your exact phrasing
- Section headers in the same format ("// ============ SECTION ==========")
- TODO/FIXME comments referencing your concepts
- Docstrings that paraphrase your spec

...you have potential copyright + §1202 claims.

**3. Error message strings** — These are gold. AI tends to copy error strings verbatim or near-verbatim. Decompile their binary, extract strings, grep for matches:
```bash
strings competitor-binary.exe | grep -i "field\|table\|pabgh\|pamt\|paz"
```

Any matches with your error messages = direct copying claim.

**4. Test fixture reuse** — If they ship test files with the same names, same JSON examples, same modder fixtures from your repo, that's verbatim copying.

**5. Variable naming idiosyncrasies** — Your unique variable names (`pabgh_bounded`, `tail_pad`, `extra_entries`, `core_block`) are distinctive enough that an independent implementation wouldn't replicate them. AI-assisted reimplementation often retains these names.

### How to gather evidence

**Step 1 — Check their public AI usage:**
- Discord/Twitter posts mentioning Claude, ChatGPT, Cursor, Copilot
- README references to "AI-assisted development"
- Commit messages mentioning AI tools
- Posts asking AI for help that reference DMM/dmm-parser

**Step 2 — Check repository traffic on YOUR repos:**
- GitHub Insights → Traffic → Clones — look for spikes around when CDUMM started
- GitHub Insights → Traffic → Visitors — country/timing patterns
- This won't show usernames but shows access volume

**Step 3 — Check for AI tool signatures:**
- GitHub Copilot generates distinctive comment styles
- ChatGPT/Claude tend to over-document with explanatory comments
- Aider produces structured commits with specific patterns
- Cursor leaves `.cursorrules` files in repos

**Step 4 — Compare structural similarity quantitatively:**
Tools like:
- `codequery` for cross-repo function-name similarity
- `tlsh` for fuzzy hashing of binaries
- `diff` on decompiled output
- AST-level comparison via tree-sitter

### Filing the claim

When you have evidence, file based on whichever claim is strongest:

1. **Verbatim string/code match** → DMCA copyright claim (always strongest)
2. **Stripped headers but identical structure** → §1202 + copyright derivative claim
3. **Architectural similarity + proven AI usage** → CDMTL §4.9 contract breach + §4.10 AI-mediated access
4. **Just structural similarity, no AI proof** → weak — focus on community/branding instead

### Honest limit on AI-mediated enforcement

The legal theories here (acceptance by access, AI as agent, imputation of knowledge) are **legally sound but largely UNTESTED in court**. No reported case yet has held a defendant liable for using AI to read GPL'd code and build a competitor. The precedents are:

- **General agency law** (Restatement (Third) of Agency) — supports holding principals liable for agents' actions
- **ProCD v. Zeidenberg (1996)** — supports browsewrap with reasonable notice
- **Specht v. Netscape (2002)** — limits browsewrap when notice is hidden
- **Field v. Google (2006)** — search/cache access without notice = no acceptance

What this means: your license clauses are **enforceable in theory** with proper notice (per-file headers), but you'll be **making law** rather than relying on it if you litigate. This means:
- Strong settlement leverage (defendants don't want to be the test case)
- Risk of unfavorable precedent if you lose
- Worth pursuing for clear violations, but pick your battles carefully

For the vast majority of CDUMM-class threats, the practical leverage comes from:
1. Per-file CMI removal (always actionable)
2. Verbatim string matches in decompiled binary (always actionable)
3. Trademark on names (actionable with registration)
4. Community/brand pressure (most effective regardless of legal status)

---

## When The Code Is Different But The Method Is The Same

This is the hardest scenario and the most important one to be honest about.

### The legal reality

**17 U.S.C. § 102(b)** explicitly excludes methods, processes, and systems from copyright protection. If a competitor:
- Wrote completely original code from scratch
- Implements the same functional approach (parse JSON → apply field intents → rewrite archives)
- Achieves the same end result as DMM

...there is **no copyright claim available**. NexusMods will not take it down on copyright grounds. Bad-faith DMCA filings can backfire under §512(f) and create liability for YOU.

This is settled law (Baker v. Selden, Computer Associates v. Altai, Sega v. Accolade, Oracle v. Google). It's not a loophole; it's the deliberate design of copyright law — methods belong in patents, not copyright.

### What you DO have leverage on

Even when code is different, these claims can still work:

**1. Contract Breach (CDMTL §4.9 — No Competing Implementation)**
If the competitor ever cloned dmm-parser, downloaded DMM, or read your Field JSON v3.1 spec doc, they accepted CDMTL by doing so. §4.9 prohibits them from building a competing tool for 3 years.

Evidence to gather:
- GitHub clone events visible in repo Insights → Traffic
- Discord/forum posts where they referenced your work
- Their own public statements about studying DMM
- Wayback Machine captures showing they viewed your repo
- Email threads where they asked you questions about DMM

If you can prove they touched your work, you have a contract claim — NOT a DMCA, but a breach of license suit. NexusMods may honor this if you provide clear evidence.

**2. Spec Document Copyright**
The Field JSON v3.1 specification text itself is copyrighted as a literary work. If their tool's documentation:
- Quotes from your spec
- Paraphrases your spec structure
- References specific section numbers from your spec
- Uses the same examples or test fixtures

...you have a derivative work claim on the documentation. This is separate from "they implement the same format."

**3. Trademark on "Field JSON v3.1"**
If their tool advertises "Compatible with Field JSON v3.1" or uses the name in their UI/documentation, that's a trademark claim — they're using your branded format name without authorization. Get the trademark registered and you can DMCA platform listings that use this term.

**4. Compatibility Claims as Source Identification**
If they say "Works with DMM mods" — and that statement is misleading or creates user confusion — that's potential false advertising under the Lanham Act §43(a) (15 U.S.C. § 1125). Different from straight trademark; doesn't require registration.

### What you DO NOT have leverage on

Be honest with yourself about these:

| Scenario | Legal Status |
|---|---|
| They wrote a parser for the same .paz files independently | Legal — file formats aren't copyrightable |
| They reverse-engineered Crimson Desert without ever touching DMM | Legal — clean-room RE is fair use |
| They built a Tauri app that mounts mods to game directories | Legal — UI patterns aren't protected |
| They use JSON to describe modifications | Legal — the IDEA of JSON-based mods isn't owned by anyone |
| They have a similar architecture | Generally legal under Altai test |
| Their tool is "inspired by" DMM but built fresh | Legal unless they accepted your license first |

### The realistic enforcement strategy for CDUMM-class threats

**Step 1 — Establish whether they ever accepted CDMTL.**
- Check your repo traffic — did their GitHub username clone dmm-parser?
- Check Discord/Reddit — did they post screenshots of DMM internals?
- Check their early commits — do they reference your spec versions or use your terminology?

If YES → §4.9 contract breach claim. File based on contract, not copyright.
If NO → you have minimal legal recourse on functionality. Pivot to community/branding.

**Step 2 — Audit their tool for ANY copying.**
- Decompile their binary, search for strings from dmm-parser
- Check if their JSON examples match yours byte-for-byte
- Check if their documentation phrases match yours
- Check if they shipped any of your test fixtures
- Check for stripped copyright headers (§1202)

ANY hit here → DMCA on copyright grounds (the strongest claim type).

**Step 3 — Evaluate trademark angles.**
- Does their tool name confuse users? (CDUMM ↔ DMM)
- Do they use "Field JSON v3.1" branding without authorization?
- Are they implying compatibility/endorsement falsely?

If yes → trademark complaint to NexusMods (separate from DMCA).

**Step 4 — Brand-level enforcement.**
This is where you'll get the most mileage:
- Establish "DMM Recognized" certification — public list of approved tools
- Publicly identify CDUMM as "unofficial / not recommended / may corrupt saves"
- Get NexusMods curators to flag unsupported tools
- Coordinate with mod authors to publish only for recognized platforms
- Use your README, NexusMods page, Discord, and social media to clarify which tool is canonical

This is how Vortex, MO2, OpenIV, and Frosty maintained dominance — not lawsuits, but community recognition. It's slower than a takedown but more durable.

**Step 5 — Last resort: full lawsuit.**
If §4.9 was clearly violated (proven prior access + competing tool), and the platform won't act on contract breach alone, you can file in federal court for breach of license + injunction. Cost: $20,000-$100,000 minimum. Generally only worth it if there's significant commercial damage.

### What I'd actually recommend

For a hobbyist modding community where CDUMM is the realistic threat:

1. **Today** — Add §4.9 to your published license (already in v1 draft above)
2. **Today** — Add a "DMM Recognized Tools" section to your README listing only DMM and SWISS
3. **This week** — Audit any current "CDUMM" or competitor for §1202 violations and code copying
4. **This month** — Register copyright on dmm-parser source + Field JSON v3.1 spec ($45-65)
5. **This quarter** — File trademark on "DMM" (or "Definitive Mod Manager") + "Field JSON v3.1" if available ($250-700)
6. **Ongoing** — Build community recognition: Discord announcements, NexusMods page text, mod author coordination

The legal protections give you takedown ammunition for clear copying. The brand/community work gives you durable dominance even when methods get cloned legally.

---

## What To Do About "CDUMM" (or any specific competitor)

If "CDUMM" or any other tool is using your work, here's the analysis path:

### Step A: Determine What Was Actually Copied

Don't assume — verify. Download CDUMM. Decompile if it's binary. Check:

1. **Does it ship dmm-parser binaries verbatim?** → Strong DMCA claim (binary copy)
2. **Does it ship modified dmm-parser source?** → Strong DMCA claim (derivative work without source disclosure)
3. **Does it parse Field JSON v3.1 files?** → Weak copyright claim (formats aren't copyrightable) BUT...
4. **Does it ship the Field JSON v3.1 SPEC DOC?** → Strong DMCA (the spec text is copyrighted as a literary work)
5. **Did they remove your copyright headers?** → §1202 claim (statutory damages)
6. **Is "CDUMM" confusingly similar to "DMM"?** → Trademark claim (especially if you register DMM™)
7. **Did they reverse-engineer your tool from scratch with no copying?** → Weak claim, generally legal

### Step B: Pick The Strongest Claim

Don't kitchen-sink the DMCA. Pick the cleanest violation:

- Best case: They literally shipped your code → file straight DMCA, 95% takedown success
- Good case: They shipped your spec doc → file DMCA on the spec, 80% success
- OK case: They stripped your CMI → §1202 claim, 70% success but smaller damages
- Marginal: They built compatible parser independently → trademark claim only on the name

### Step C: Send The Notice and Document Outcome

Use the template above. Track the result. If they counter-notice, you have a decision: litigate or let it stay up.

### Step D: If You Can't Take It Down — Public Pressure

If your DMCA fails or counter-notice succeeds and you can't litigate:
- Update your README to publicly identify CDUMM as "unofficial / not authorized"
- Post on the Crimson Desert subreddit clarifying which tool is canonical
- Use NexusMods page descriptions to direct users to authorized tools
- This isn't legal enforcement but it's social-layer enforcement that often works in modding communities

---

## Realistic Expectations

**What this license + playbook GETS YOU:**
- A clear basis for DMCA takedowns of binary/source/spec copying — high success rate
- §1202 leverage for stripped attribution — strong settlement tool
- Trademark protection for product names (when registered)
- Public-facing legal stance that discourages bad actors from starting trouble
- Ammunition for community-side reputation fights

**What this license + playbook DOES NOT get you:**
- Unilateral power to remove competitors NexusMods deems independent works
- Enforcement of "Authorized Suite only" use restrictions on third parties (that's lawsuit territory)
- Protection against clean-room reverse engineering
- Ability to copyright the file format itself (Section 102(b) of the Copyright Act)
- Government enforcement without you initiating it

**Bottom line:** The license + this playbook give you a 70-85% effective enforcement toolkit on platforms like NexusMods. The remaining 15-30% requires either a lawsuit or community pressure. That's the realistic ceiling for any license — even Microsoft, Adobe, and Oracle face the same enforcement gaps.

---

## Quick Reference: Files To Update Before Going Live

When you adopt CDMTL v1.0, update these files:

- [ ] `LICENSE.txt` (or `LICENSE`) in every RicePaddySoftware repo — replace MPL-2.0 with CDMTL v1.0
- [ ] `README.md` in every repo — add "Licensed under CDMTL v1.0" + Authorized Channels list
- [ ] Per-file headers in all `.rs`, `.py`, `.ts` files — add SPDX + copyright + CMI notice
- [ ] NexusMods mod page descriptions — add license badge + "unauthorized copies will be DMCA'd" notice
- [ ] GitHub repository "About" section — add license link
- [ ] `Cargo.toml` `license = "LicenseRef-CDMTL-1.0"` (use LicenseRef- because CDMTL isn't in SPDX)
- [ ] `pyproject.toml` `license = { text = "CDMTL-1.0" }`
- [ ] Python package `setup.py` if applicable
- [ ] CHANGELOG.md noting the relicense from MPL-2.0 → CDMTL v1.0

---

*End of Playbook. Pair this with `LICENSE_DRAFT_v1.md` for the operative legal text.*

---

## Nexus Mods Distribution Kit

# NexusMods Enforcement Kit

The minimum viable kit for getting unauthorized copies removed from NexusMods. Cut down from the full enforcement playbook — this is the operational stuff only.

---

## The Honest Reality

**NexusMods staff will NOT read your license.** They will not evaluate clauses. They handle DMCA notices and trademark complaints based on **clear evidence**, nothing else.

So your license is mostly irrelevant to the takedown decision. What matters:

1. **Authorship proof** — can you show you made it first?
2. **Clear copying** — can you show their upload matches yours?
3. **A clean DMCA notice** — properly formatted, all elements present?

That's it. The license you wrote, the per-file headers, the §4.9/§4.10 clauses — those come into play only if you sue (which you said you won't). For NexusMods, it's: **prove you made it, prove they copied it, send a proper DMCA.**

---

## Three Things You Need (in priority order)

### 1. Public "Authorized Channels" notice (DONE — in your READMEs)

Without this, NexusMods staff have no way to verify "this upload is unauthorized." With it, you have a clean basis for every takedown: "It's not on the authorized channels list."

This is now in:
- `dmm-parser/README.MD`
- `dmm-api-test/README.md` (DMM-BETA)
- `CRIMSON-DESERT-SAVE-EDITOR-AND-GAME-MODS-clone/README.md` (SWISS)

When you publish DMM on NexusMods, **paste the same Authorized Channels notice into the mod page description**. That's where NexusMods staff will look first when evaluating a complaint.

### 2. Trademark filing for "DMM" or "Definitive Mod Manager" — $250–350, do this NOW

This is the single highest-leverage action. With a registered ® on "DMM" or "Definitive Mod Manager":

- Any tool calling itself "CDUMM" / "DUMM" / "DMM2" is a trademark complaint, not a copyright claim
- NexusMods has a separate trademark complaint process that's faster than DMCA
- You don't need to prove copying — just the confusing similarity
- Statutory damages start at $1,000 per counterfeit good

**How:**
1. Go to https://www.uspto.gov/trademarks/apply
2. Use TEAS Plus ($250) — cheapest option
3. File for: "Definitive Mod Manager" in **Class 9** (Computer software)
4. Description: "Computer software for installing, managing, and applying modifications to video games"
5. Specimen: screenshot of DMM running with the name visible
6. Wait 6–12 months for registration. **TM symbol can be used immediately**, ® only after registration.

**If you want to spread the budget:**
- Priority 1: "Definitive Mod Manager" — $250 (1 mark, Class 9)
- Priority 2: "Field JSON v3.1" — could be tricky (descriptive marks get rejected)
- Priority 3: "SWISS Suite" — $250 (1 mark, Class 9)

Total recommended: $500 covers the two most useful marks.

### 3. Pre-filled DMCA template (BELOW — copy/paste/fire when needed)

You don't have time to write a DMCA from scratch when you find a violation. Have it ready to send.

---

## DMCA Template (Copy/Paste Ready)

**Send to:** `dmca@nexusmods.com`

**Subject line:** `DMCA Takedown Notice — Unauthorized Copy of [TOOL NAME] — [INFRINGING URL or USERNAME]`

**Body:**

```
To: NexusMods DMCA Agent

I am the copyright owner of [TOOL NAME], distributed by RicePaddySoftware
under the Crimson Desert Modding Tools License v1.0 (CDMTL v1.0).

This notice complies with the Digital Millennium Copyright Act
17 U.S.C. § 512(c)(3).

1. IDENTIFICATION OF COPYRIGHTED WORK:
   Tool: [DMM / SWISS Save Editor / dmm-parser / etc.]
   Original distribution URL: https://github.com/exodiaprivate-eng/[REPO]
   First publication date: [GIT FIRST COMMIT DATE]
   License: CDMTL v1.0 — https://github.com/exodiaprivate-eng/[REPO]/blob/main/LICENSE.txt
   [Include if registered:] US Copyright Registration: TX-[NUMBER]

2. IDENTIFICATION OF INFRINGING MATERIAL:
   URL: [FULL NEXUSMODS URL]
   Mod name: [as displayed on the page]
   Uploader username: [their NexusMods username]
   Date uploaded: [date from the page]

3. EVIDENCE OF INFRINGEMENT:
   [Pick the strongest one or two — do not list all unless they all apply]

   (a) Direct binary copy. SHA-256 of my official release [VERSION]:
       [HASH]
       SHA-256 of their uploaded file (downloaded [DATE]):
       [HASH]
       The hashes match — this is a verbatim redistribution of my software
       without authorization.

   (b) Source code copying. The uploaded archive contains source files
       directly copied from my repository, including [SPECIFIC FILES].
       Side-by-side comparison attached.

   (c) Stripped Copyright Management Information. The uploaded copy has
       had the per-file copyright headers (which were present in my
       original distribution) removed or altered. This independently
       violates 17 U.S.C. § 1202.

   (d) Distribution outside Authorized Channels. My software is
       distributed exclusively through the authorized channels listed
       at https://github.com/exodiaprivate-eng/[REPO]#license--authorized-distribution.
       This NexusMods upload is by an unauthorized party and is not part
       of those channels.

4. GOOD FAITH STATEMENT:
   I have a good faith belief that use of the material in the manner
   complained of is not authorized by the copyright owner, its agent,
   or the law.

5. ACCURACY STATEMENT:
   The information in this notification is accurate, and under penalty
   of perjury, I am authorized to act on behalf of the copyright owner
   of the exclusive right that is allegedly infringed.

6. CONTACT INFORMATION (required by 17 U.S.C. § 512(c)(3)(A)(iv)):
   Name: [YOUR LEGAL NAME]
   Affiliation: RicePaddySoftware (Copyright Owner)
   Email: [YOUR EMAIL]
   Postal address: [YOUR PHYSICAL ADDRESS — DMCA REQUIRES THIS]
   Phone: [YOUR PHONE — DMCA REQUIRES THIS]

7. SIGNATURE:
   /s/ [YOUR NAME]
   Date: [TODAY'S DATE]

ATTACHMENTS:
- Screenshot of infringing NexusMods page (saved with timestamp)
- SHA-256 of original release: [from your records]
- SHA-256 of downloaded infringing file
- Side-by-side comparison of any code or assets, if applicable
- Link to original LICENSE.txt establishing CDMTL v1.0 terms
```

### What goes in the placeholders

Before you ever need this, fill in your fixed info ONCE:

```
[YOUR LEGAL NAME] = [fill in]
[YOUR EMAIL] = [fill in]
[YOUR PHYSICAL ADDRESS] = [fill in — DMCA REQUIRES this]
[YOUR PHONE] = [fill in — DMCA REQUIRES this]
```

**Keep this somewhere safe.** A DMCA without contact information will be rejected.

---

## Trademark Complaint Template (Different Process)

For "CDUMM" / "DUMM" / "DMM2" / similar-name copies, file a TRADEMARK complaint, not a DMCA.

**Send to:** `legal@nexusmods.com` (NOT `dmca@`)

**Subject:** `Trademark Complaint — Unauthorized use of "[YOUR MARK]" — [INFRINGING URL]`

**Body:**

```
To: NexusMods Legal Team

I am writing on behalf of RicePaddySoftware regarding unauthorized
use of our trademark on a NexusMods upload.

1. TRADEMARK INFORMATION:
   Mark: [DMM / Definitive Mod Manager / Field JSON v3.1 / etc.]
   Owner: RicePaddySoftware
   Registration status: [USPTO Reg. No. XXXXXXX / Pending application Serial No. XXXXXXX / Common-law trademark since FIRST USE DATE]
   Use in commerce: Computer software for Crimson Desert game modification
   Class: 9 (Computer software)

2. INFRINGING USE:
   URL: [FULL NEXUSMODS URL]
   Mod name: [as displayed]
   Uploader: [username]
   Manner of infringement:
   - The uploader names their tool "[INFRINGING NAME]" which is
     confusingly similar to our registered mark "[YOUR MARK]"
   - This creates likelihood of confusion in the Crimson Desert
     modding community as to the source, sponsorship, or affiliation
     of the upload
   - The infringing tool targets the same user base (Crimson Desert
     mod users) and serves the same function (mod management /
     [whatever])
   - Our trademark predates this upload by [TIMEFRAME]

3. RELIEF REQUESTED:
   Removal of the infringing upload, or at minimum, requirement that
   the uploader rename their tool to remove the confusingly similar
   mark.

4. CONTACT:
   Name: [YOUR NAME]
   Affiliation: RicePaddySoftware
   Email: [YOUR EMAIL]
   Phone: [YOUR PHONE]

ATTACHMENTS:
- Screenshot of infringing page
- Evidence of trademark registration (USPTO TSDR printout) or
  evidence of first use in commerce (early commits, blog posts,
  Discord announcements with timestamps)
- Side-by-side comparison of the marks
```

---

## Evidence Collection (5 minutes, do every release)

When you ship a new version of DMM / SWISS / dmm-parser, automate this:

```bash
# After every release:
sha256sum dist/dmm-installer.exe > releases/dmm-v1.2.3.sha256
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3

# Save these in a spreadsheet:
# - Version
# - Release date
# - SHA-256 hash
# - GitHub Release URL
# - NexusMods upload URL
```

When a violation appears, you can immediately point to:
- "I released v1.2.3 on [DATE] with SHA-256 [HASH]"
- "Their upload SHA-256 is [HASH] — matches mine"

This kind of clean evidence makes NexusMods takedowns nearly automatic.

---

## What NexusMods Will and Will Not Honor

### WILL honor (high success rate)

- Direct binary/source copying with SHA-256 evidence
- Stripped copyright headers (§1202 angle)
- Trademark infringement (especially with registered ®)
- Confusingly similar names that target your user base
- Third-party reuploads of your binaries
- Stolen assets/icons/screenshots from your tool

### WILL NOT honor (skip these claims)

- "They violated my license clauses 4.9/4.10"
- "They reverse-engineered my tool"
- "They consume my JSON format"
- "Their architecture is similar to mine"
- "They built a competing tool"

If your evidence boils down to "I think they used AI to read my code," NexusMods will pass and tell you to file in court. Save those arguments for situations where you have actual code/string matches.

---

## When You're Ready to Publish DMM on NexusMods

Drop this exact text into the mod page description (right after the basic description):

```markdown
## License & Authorized Distribution

DMM is licensed under CDMTL v1.0 (https://github.com/exodiaprivate-eng/Definitive-Mod-Manager/blob/main/LICENSE.txt).

This is the ONLY authorized NexusMods upload of DMM, by the official
author RicePaddySoftware. Any other NexusMods upload of DMM, or any
similarly-named tool ("CDUMM", "DUMM", "DMM2", etc.) is unauthorized
and a trademark/copyright violation.

If you find an unauthorized copy on NexusMods, please report it via
the "Report Mod" button.
```

This single paragraph gives NexusMods staff what they need to evaluate any future complaint instantly.

---

## TL;DR — The Three Things That Matter

1. **Authorized Channels notice in README + NexusMods page** — DONE
2. **Trademark registration** ($250–500, 6–12 months wait) — DO THIS WEEK
3. **DMCA template + filled-in contact info ready to fire** — KEEP THIS DOC HANDY

That's it. Everything else is bonus.
