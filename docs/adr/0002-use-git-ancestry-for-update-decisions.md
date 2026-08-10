# Use Git Ancestry for Update Decisions

A Git Skill Source records its repository URL, Skill subpath, tracked branch and installed commit OID. Skill Deck compares the tracked branch's remote HEAD with the installed commit and treats only fast-forward ancestry as an ordinary update; tags and Skill metadata versions are display-only because Agent Skills defines no version protocol.
