import json, os, sys, urllib.request
def call(method, **params):
    req=urllib.request.Request(os.environ.get("IDA_MCP_URL", "http://localhost:%s/mcp" % os.environ.get("IDA_MCP_PORT", "13337")),
        data=json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":params}).encode(),
        headers={"Content-Type":"application/json"})
    r=json.load(urllib.request.urlopen(req, timeout=120))
    if "error" in r: raise RuntimeError(r["error"])
    return r["result"]
if __name__=="__main__":
    out=call(sys.argv[1], **json.loads(sys.argv[2] if len(sys.argv)>2 else "{}"))
    s=out if isinstance(out,str) else json.dumps(out,indent=1,ensure_ascii=False)
    sys.stdout.buffer.write(s.encode("utf-8","replace"))
