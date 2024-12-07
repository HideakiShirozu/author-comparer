import { hydrate, prerender as ssr } from 'preact-iso'
import { css } from '../styled-system/css'
import './index.css'
import { useState } from 'preact/hooks'

import ResultViewer from './components/ResultViewer'

interface CompareResult {
	same_author: boolean
	confidence: number
	detailed_analysis: Array<{
		aspect: string
		difference: number
		explanation: string
	}>
}

function CompareField({
	callback,
	title,
}: { callback: (newText: string) => void; title: string }) {
	return (
		<div
			class={css({
				width: '100%',
				display: 'flex',
				flexDirection: 'column',
				gap: '8px',
				sm: {
					width: '50%',
				},
				color: '#1a1a1a',
			})}
		>
			<h2
				class={css({
					fontSize: '16px',
					fontWeight: 'bold',
					color: '#2D3748',
				})}
			>
				{title}
			</h2>
			<textarea
				class={css({
					width: '100%',
					height: '300px',
					padding: '12px',
					borderRadius: '8px',
					border: '1px solid #E2E8F0',
					fontSize: '14px',
					lineHeight: '1.5',
					resize: 'none',
					_focus: {
						outline: 'none',
						borderColor: '#4299E1',
						boxShadow: '0 0 0 1px #4299E1',
					},
					'&::-webkit-scrollbar': {
						width: '8px',
					},
					'&::-webkit-scrollbar-track': {
						background: '#EDF2F7',
						borderRadius: '4px',
					},
					'&::-webkit-scrollbar-thumb': {
						background: '#CBD5E0',
						borderRadius: '4px',
						_hover: {
							background: '#A0AEC0',
						},
					},
				})}
				onChange={(e) => callback((e.target as HTMLTextAreaElement).value)}
				placeholder={`Enter ${title.toLowerCase()} here...`}
			/>
		</div>
	)
}

export function App() {
	const [text1, setText1] = useState('')
	const [text2, setText2] = useState('')
	const [result, setResult] = useState<CompareResult | null>(null)
	const [loading, setLoading] = useState(false)

	const handleSubmit = async () => {
		setLoading(true)
		try {
			const response = await fetch('http://localhost:8000/compare', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					text1,
					text2,
				}),
			})
			const data = await response.json()
			setResult(data)
		} catch (e) {
			console.error('Error:', e)
			alert('An error occurred while comparing texts.\nPlease ensure the API server is running.')
		} finally {
			setLoading(false)
		}
	}

	return (
		<div
			class={css({
				minHeight: '100vh',
				backgroundColor: '#F7FAFC',
				padding: '32px',
			})}
		>
			<div
				class={css({
					maxWidth: '1200px',
					margin: '0 auto',
				})}
			>
				<h1
					class={css({
						fontSize: '32px',
						fontWeight: 'bold',
						color: '#1A202C',
						marginBottom: '24px',
						textAlign: 'center',
					})}
				>
					Author Comparer
				</h1>
				<div
					class={css({
						backgroundColor: 'white',
						padding: '24px',
						borderRadius: '12px',
						boxShadow:
							'0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
					})}
				>
					<div
						class={css({
							display: 'flex',
							justifyContent: 'center',
							flexDirection: {
								base: 'column',
								sm: 'row',
							},
							gap: '24px',
							marginBottom: '24px',
						})}
					>
						<CompareField callback={setText1} title="First Text" />
						<CompareField callback={setText2} title="Second Text" />
					</div>
					<button
						type="submit"
						onClick={handleSubmit}
						disabled={loading || !text1 || !text2}
						class={css({
							width: '100%',
							padding: '12px',
							backgroundColor:
								loading || !text1 || !text2 ? '#CBD5E0' : '#4299E1',
							color: 'white',
							fontWeight: 'bold',
							borderRadius: '8px',
							cursor: loading || !text1 || !text2 ? 'not-allowed' : 'pointer',
							_hover: {
								backgroundColor:
									loading || !text1 || !text2 ? '#CBD5E0' : '#3182CE',
							},
							transition: 'background-color 0.2s',
						})}
					>
						{loading ? 'Analyzing...' : 'Compare Texts'}
					</button>
				</div>
				{result && <ResultViewer result={result} />}
			</div>
		</div>
	)
}

if (typeof window !== 'undefined') {
	hydrate(<App />, document.getElementById('app'))
}

export async function prerender(data) {
	return await ssr(<App {...data} />)
}
