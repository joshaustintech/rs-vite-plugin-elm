# Security Watchlist

The post-action review must treat these 100 classes as the minimum review floor.
Confirmation still requires exact file and code path, attacker-controlled input, sink or broken control, reachable preconditions, plausible impact, and a reproducible proof or test.

1. command injection
2. shell injection
3. argument injection
4. path traversal
5. directory traversal
6. zip slip
7. arbitrary file write
8. arbitrary file read
9. arbitrary file delete
10. symlink race
11. hardlink race
12. temp-file race
13. TOCTOU
14. SSRF
15. DNS rebinding
16. open redirect
17. reflected XSS
18. stored XSS
19. DOM XSS
20. HTML injection
21. JS injection
22. template injection
23. expression injection
24. CSS injection
25. JSON injection
26. SQL injection
27. NoSQL injection
28. LDAP injection
29. XPath injection
30. XXE
31. XML entity expansion
32. unsafe deserialization
33. prototype pollution
34. object injection
35. format string injection
36. log injection
37. header injection
38. CRLF injection
39. response splitting
40. request smuggling
41. auth bypass
42. authz bypass
43. IDOR
44. tenant escape
45. privilege escalation
46. session fixation
47. session hijacking
48. CSRF
49. CORS misconfig
50. replay attack
51. password reset bypass
52. MFA bypass
53. missing re-auth on sensitive action
54. insecure direct state change
55. debug endpoint exposure
56. verbose error leak
57. source map disclosure
58. secret leakage in logs
59. hardcoded credentials
60. insecure randomness
61. predictable tokens
62. weak hash use
63. hash collision abuse
64. cache poisoning
65. resource exhaustion
66. CPU exhaustion
67. memory exhaustion
68. thread exhaustion
69. stack overflow
70. recursion overflow
71. deadlock
72. livelock
73. race condition
74. data race
75. integer overflow
76. integer underflow
77. out-of-bounds read
78. out-of-bounds write
79. use after free
80. double free
81. null pointer deref
82. panic on untrusted input
83. unwrap on untrusted input
84. expect on untrusted input
85. unchecked indexing
86. unchecked UTF-8 conversion
87. unsafe FFI boundary
88. unsafe code block
89. buffer overflow
90. path disclosure
91. build artifact overwrite
92. environment variable poisoning
93. process path poisoning
94. arbitrary module resolution
95. import path confusion
96. canonicalization bypass
97. Unicode normalization bypass
98. URL decoding bypass
99. HMR event injection
100. stale dependency graph confusion
