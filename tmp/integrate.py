import re, sys
PATH = 'SPECIFICATION.md'
with open(PATH, 'r', encoding='utf-8') as f:
    text = f.read()
marker = '# Revision 0.5 Addendum'
if marker not in text:
    sys.exit('addendum marker not found')
body, addendum = text.split(marker, 1)
lines = body.rstrip().split('\n')
while lines and (lines[-1].strip() in ('', '---') or
                 lines[-1].strip().startswith('*End of specification, Revision 0.4')):
    lines.pop()
body = '\n'.join(lines) + '\n'
sections = {}
pattern = re.compile(r'^## \xc2\xa7?([0-9.+]+)\s+([^\n]+)\n'.encode().decode('unicode_escape'), re.MULTILINE)
pattern = re.compile(r'^## §([0-9.+]+)\s+([^\n]+)\n', re.MULTILINE)
matches = list(pattern.finditer(addendum))
for i, m in enumerate(matches):
    key = m.group(1).rstrip('.')
    start = m.end()
    end = matches[i + 1].start() if i + 1 < len(matches) else len(addendum)
    sections[key] = addendum[start:end].rstrip() + '\n'
expected = {'1.5', '4.4.7', '4.4.8', '4.4.9', '4.4.10', '4.7+', '6.7', '20'}
missing = expected - set(sections)
if missing:
    sys.exit(f'missing addendum sections: {missing}')
sections['20'] = re.sub(r'\n---\n\n\*End of specification[^\n]*\*\s*$', '\n', sections['20']).rstrip() + '\n'
body = body.replace('Engineering Specification, Revision 0.5', 'Engineering Specification, Revision 0.6', 1)
new_changelog = '''## What changed in Revision 0.6

Revision 0.6 is a **re-integration pass**. No new content; rev-0.5's
addendum sections are now woven into their logical positions:

- §1.5 *Minimum Viable Gildos* is now part of §1.
- §4.4 lists nine AI-native primitives in order (the rev-0.5 trio
  becomes §4.4.7, §4.4.8, §4.4.9; the syscall summary becomes §4.4.10).
- §4.7 conformance suites for the rev-0.5 primitives are inline.
- §6.7 *End-to-end walkthrough* is now part of §6.
- §20 *Glossary* is a numbered top-level section.
- The standalone "Revision 0.5 Addendum" trailer is removed.

The body of the spec is otherwise byte-identical to rev 0.5.

---

'''
body = body.replace('## What changed in Revision 0.5', new_changelog + '## What changed in Revision 0.5', 1)
old_toc = '''## Table of Contents

1.  Executive Summary
2.  Principles and Enforced Choices
3.  Architecture at a Glance
4.  **The Gildos Kernel**
5.  Boot, Lifecycle, and Recovery
6.  Cognitive Runtime (`cogd`)
7.  Semantic Event Bus
8.  Capability System
9.  Semantic Memory
10. Self-Improvement Loop
11. Continuous Learning and Skill Acquisition
12. Security Model
13. Storage and Persistence
14. Networking, Identity, and Federation
15. Generated Drivers
16. Target Hardware
17. Testing and Development Strategy
18. Roadmap and Prerequisites
19. Open Research Questions'''
new_toc = '''## Table of Contents

1.  Executive Summary
    - 1.5 Minimum Viable Gildos (the 90-day MVP)
2.  Principles and Enforced Choices
3.  Architecture at a Glance
4.  **The Gildos Kernel** — nine AI-native primitives
5.  Boot, Lifecycle, and Recovery
6.  Cognitive Runtime (`cogd`)
    - 6.7 End-to-end walkthrough (one user turn)
7.  Semantic Event Bus
8.  Capability System
9.  Semantic Memory
10. Self-Improvement Loop
11. Continuous Learning and Skill Acquisition
12. Security Model
13. Storage and Persistence
14. Networking, Identity, and Federation
15. Generated Drivers
16. Target Hardware
17. Testing and Development Strategy
18. Roadmap and Prerequisites
19. Open Research Questions
20. Glossary'''
if old_toc not in body:
    sys.exit('TOC block not found verbatim')
body = body.replace(old_toc, new_toc, 1)
insert_15 = '\n### 1.5 Minimum Viable Gildos (90-day MVP)\n\n' + sections['1.5'] + '\n'
marker_2 = '\n## 2. Principles and Enforced Choices'
if marker_2 not in body:
    sys.exit('section 2 marker not found')
body = body.replace(marker_2, insert_15 + '---\n' + marker_2, 1)
i1 = body.find('#### 4.4.7 Summary of new syscalls')
i2 = body.find('### 4.5 Kernel ABI versioning')
if i1 == -1 or i2 == -1 or i1 > i2:
    sys.exit('section 4.4.7 / 4.5 markers not found')
new_44 = (
    '#### 4.4.7 Cognitive Fork (`kvfork`)\n\n' + sections['4.4.7'] + '\n'
    '#### 4.4.8 Persistent Prompt Objects (`promptpin`)\n\n' + sections['4.4.8'] + '\n'
    '#### 4.4.9 Cognitive cgroups (`cogcg`)\n\n' + sections['4.4.9'] + '\n'
    '#### 4.4.10 Summary of new syscalls\n\n' + sections['4.4.10'] + '\n'
)
body = body[:i1] + new_44 + body[i2:]
i47 = body.find('### 4.7 Testing the kernel layer')
i48 = body.find('### 4.8 Why not a custom kernel in Phase 1?')
if i47 == -1 or i48 == -1 or i47 > i48:
    sys.exit('section 4.7 / 4.8 markers not found')
existing_47 = body[i47:i48]
gate = 'These are gating tests for any kernel-module change. Failure means the\nchange does not merge.'
if gate not in existing_47:
    sys.exit('gate sentence not found')
new_47 = existing_47.replace(gate, sections['4.7+'].strip() + '\n\n' + gate, 1)
body = body[:i47] + new_47 + body[i48:]
m7 = '\n## 7. Semantic Event Bus'
if m7 not in body:
    sys.exit('section 7 marker not found')
body = body.replace(m7, '\n### 6.7 End-to-end walkthrough — one user turn\n\n' + sections['6.7'] + '\n---\n' + m7, 1)
body = re.sub(r'\n---\n\n\*End of specification[^\n]*\*\s*$', '', body).rstrip() + '\n'
body += '\n---\n\n## 20. Glossary\n\n' + sections['20'].lstrip() + '\n---\n\n*End of specification, Revision 0.6.*\n'
with open(PATH, 'w', encoding='utf-8') as f:
    f.write(body)
print('OK')
