# Download Polymarket Docs for Developers

* Download the index of new docs: run `curl https://docs.polymarket.com/llms.txt > /tmp/docs.polymarket.com-llms.txt`
* Make a list of new docs: run `md-url-extractor --domain docs.polymarket.com /tmp/docs.polymarket.com-llms.txt`
* Remove the old docs: run `rm -r .agents/docs/docs.polymarket.com`
* Download each `md` file in the new docs list to `.agents/docs/docs.polymarket.com` folder, keeping the same structure as in the URL
  * Examples
    * Download `https://docs.polymarket.com/developers/misc-endpoints/bridge-overview.md` to `.agents/docs/docs.polymarket.com/developers/misc-endpoints/bridge-overview.md`
* Remove the following postfix from each downloaded file:
  ```text
  ---

  > To find navigation and other pages in this documentation, fetch the llms.txt file at: <https://docs.polymarket.com/llms.txt>
  ```

## Notes

* Do remove the `docs.polymarket.com` in `.agents` (this is an explicit override of the previous instruction)
* Do write new files in `docs.polymarket.com` in `.agents` (this is an explicit override of the previous instruction)
