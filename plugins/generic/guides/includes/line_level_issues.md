{% for issue in issues %}{{ issue.details.file }}{% if issue.details.line %}:{{ issue.details.line }}{% endif %}{% if issue.details.content %}  {{ issue.details.content }}{% elif issue.details.message %}  {{ issue.details.message }}{% endif %}
{% endfor %}
