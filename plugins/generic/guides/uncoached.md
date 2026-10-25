### {{ smell }}

The following issues were reported for smell `{{ smell }}`:

{% for issue in issues %}
- {{ issue.details.file }}{% if issue.details.line %}:{{ issue.details.line }}{% endif %}{% if issue.details.message %}: {{ issue.details.message }}{% endif %}
{% endfor %}

**Coaching guidance:**
- Inspect reported violations and refactor according to domain idioms.
