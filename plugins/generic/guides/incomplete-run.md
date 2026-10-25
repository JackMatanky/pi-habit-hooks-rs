⚠️ Habit Hooks: Incomplete Run

Automated checks could not run to completion:

{% for issue in issues %}
- {{ issue.key }}{% if issue.details.message %}: {{ issue.details.message }}{% endif %}
{% endfor %}
