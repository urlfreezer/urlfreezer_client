# Simple Client Library & Cli for URL Freezer


As today the main functionality is allow to fetch URL Freezer  links using a list 
of links from a CSV file.

Example Source File:
```csv
page,link,label
"https://your_site_example.com/post/good.html",https://external_example.com/yes_is_good.html,Good
https://your_site_example.com/post/great.html,https://other_example.com/quite_great.html,Great
```

Example command
```sh
urlfreezer --user_id ufu_ID_FROM_URLFREEZER  < links.csv 
```

Example Output

```
page,original,label,link,action
https://your_site_example.com/post/good.html,https://external_example.com/yes_is_good.html,Good,https://urlfreezer.com/wtcdrdsirermsesg,Redirect
https://your_site_example.com/post/great.html,https://other_example.com/quite_great.html,Great,https://urlfreezer.com/wtwdserkfisrtkjf,Content
```




