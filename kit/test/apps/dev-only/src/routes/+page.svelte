<script>
	import { base, assets } from '$app/paths';
</script>

<div>
<img src="" alt="Swirl colors top graphic" width="100" height="100" loading="lazy">
</div>

<div class="centered">
<h1>TEE contract offchain demo</h1>

<h2>Start</h2>
<p>From:</p> <!--filled in by wallet-->
<p>Balance:</p> <!--filled in by wallet-->
<p>To begin using Secret Network, connect your Keplr wallet.</p>

<div id="content">
<h2>Send</h2>

<form method="POST"> <!--need server-side action and js if-return statements-->
  <label for="addr_to">
    To:
      <input type="text" id="addr_to" name="addr_to">
      </input>
  </label><br><br>
  <label for="amt_to">
    Amount:
      <input type"number" id="amt_to" name="amt_to">
      </input>
  </label><br>
  <label for="memo_to">
    Memo:
      <input type"text" id="memo_to" name="memo_to" size="32">
      </input>
  </label>
</form>

<p>Click the "Submit" button to send.</p>
</div>

<!-- create read only text that posts what is happening in backend-->
<!--Sequence Processed() uint seq number processed-->
<!--GET Next Received () uint since seq number-->
<!--POST Receipt () Seq number, from, amount, memo-->
</div>


<style>

.bcknd {

.centered {
		max-width: 40em;
		margin: 2em auto; 
    position: relative;
	}

	#content{
        border: .5em solid #333;
        background-color: #333366;
        margin: 2em auto;
        padding: 1em;
        position: relative;
        opacity: 30%;
        
    }
    
    label {
		width: 100%;
	}

	input {
		flex: 1;
	}

	span {
		flex: 1;
	}

	button {
		border: none;
    	background: url(./remove.svg) no-repeat 50% 50%;
		background-size: 1rem 1rem;
		cursor: pointer;
		height: 100%;
		aspect-ratio: 1;
		opacity: 0.5;
		transition: opacity 0.2s;
	}

	button:hover {
		opacity: 1;
	}

	.saving {
		opacity: 0.5;
	}
</style>


