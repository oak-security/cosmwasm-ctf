# Oak Security CosmWasm CTF ⛳️

Crack all our challenges and show the community that you know your way in security, either as an auditor or a security-minded developer! This CTF was run as a live event during AwesomWasm 2023, for info related to the event check [this other file](./awesomwasm-2023/README.md). 

Follow us on Twitter at [@SecurityOak](https://twitter.com/SecurityOak) to receive the latest news on Cosmos security and fresh audit reports. 

## Getting started

To get started with the challenges, please go to the [main](https://github.com/oak-security/cosmwasm-ctf/tree/main) branch. The 10 challenges follow no particular difficulty order, number 1 may not be easier than number 10 and the other way around. Each of them showcase a different security issue or exploitation techniques that we find during our security audits.

<table>
    <tr>
        <td>1. <a href="./ctf-01/README.md">Mjolnir</a></td>
        <td>6. <a href="./ctf-06/README.md">Hofund</a></td>
    </tr>
    <tr>
        <td>2. <a href="./ctf-02/README.md">Gungnir</a></td>
        <td>7. <a href="./ctf-07/README.md">Tyrfing</a></td>
    </tr>
    <tr>
        <td>3. <a href="./ctf-03/README.md">Laevateinn</a></td>
        <td>8. <a href="./ctf-08/README.md">Gjallarhorn</a></td>
    </tr>
    <tr>
        <td>4. <a href="./ctf-04/README.md">Gram</a></td>
        <td>9. <a href="./ctf-09/README.md">Brisingamen</a></td>
    </tr>
    <tr>
        <td>5. <a href="./ctf-05/README.md">Draupnir</a></td>
        <td>10. <a href="./ctf-10/README.md">Mistilteinn</a></td>
    </tr>
</table>

After you have given your best to solve each of the challenges, we encourage you to create an "audit-like" report. You can follow [this template](./SAMPLE_REPORT_TEMPLATE.md) or any other that you consider suitable.

Your results are ready now! we have published our own writeups so you can compare and check if your solutions are correct. Please visit:
1. [Capture The Flag ️Writeups — part 1](https://medium.com/oak-security/capture-the-flag-%EF%B8%8Fwriteups-awesomwasm-2023-pt-1-a40c6e506b49)
2. [Capture The Flag ️Writeups — part 2](https://medium.com/oak-security/capture-the-flag-%EF%B8%8Fwriteups-awesomwasm-2023-pt-2-cb3e9b297c0)

In addition:
1. To view the proof of concept for the challenges, please visit the [poc-exploit](https://github.com/oak-security/cosmwasm-ctf/tree/poc-exploit) branch. The proof of concept is written as an `exploit()` test case and can be found in the `exploit.rs` file.
2. To view the fixed versions of the challenges, please visit the [fixed](https://github.com/oak-security/cosmwasm-ctf/tree/fixed) branch. All proof of concept test cases are prefixed with `#[ignore="bug is patched"]`, so they will not be automatically executed when running `cargo test`.

### Running test cases

1. Navigate into challenge folder.

```bash
cd ctf-01/
```

2. Run tests

```bash
cargo test
```

## Questions?

Just open an issue in this repository to get an answer from our team.
