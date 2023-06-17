<script>
	import { base, assets } from '$app/paths';
</script>

<body>

<div id="swirl">
<img src="/static/swirl-top.svg" alt="Swirl colors top graphic" width="100" height="100" loading="lazy">
</img>
</div>

<div class="centered">
<h1>TEE contract offchain demo</h1>

<h2>Connect wallet</h2>
<p>From:</p> <!--filled in by wallet-->
<p>Balance:</p> <!--filled in by wallet-->
<p>To begin using Secret Network, connect your Keplr wallet.</p> <br>
<a href=""><img src="/static/keplr_logo.png" alt="Keplr logo" width="64" height="64">
</img> </a>

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

</body>

<style>

body {
 background-image: url("../static/cover.png");
 background-color: #333;
}


#swirl {
  	opacity: 0.8;
  	z-index: 1;
	position: absolute;
	max-width: 40em;

}

.centered {
		opacity: 0.2;
  		z-index: 2;
		max-width: 30em;
		margin: 2em auto; 
    	position: relative;
	}

	#content{
        opacity: 0.4;
		position: relative;
		border: .5em solid #333;
        background-color: #333366;
        margin: 2em auto;
        padding: 1em;
        
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


