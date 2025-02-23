Hides arbitrary data in plain text using unicode variation selectors.

According to the unicode standard, applications should just pass through unknown variation selectors, so the hidden data should survive most copy-pasting, uploading etc.

Some applications sadly don't correctly follow this, so with increasing amounts of hidden data, some rendering issues (replacement characters, random spaces etc.) can occur.
